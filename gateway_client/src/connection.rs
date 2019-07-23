
use futures::prelude::*;
use crate::Error;
use crate::model;
use std::time::{Duration,Instant};
use std::pin::Pin;
use std::task::{Poll,Context};
use futures::{
    sink::{Sink},
};
use std::marker::PhantomData;

use tracing::*;

use futures_01::stream::Stream as _;
use futures::compat::*;

pub struct CloseOnDrop<S: Sink<I> + Unpin,I>(S,PhantomData<I>);

impl<S,I> CloseOnDrop<S,I>
    where S: Sink<I> + Unpin,
{
    pub fn new(s: S) -> Self{
        Self(s,PhantomData)
    }
}

impl<S,I> Sink<I> for CloseOnDrop<S,I>
    where S: Sink<I> + Unpin
{
    type Error = S::Error;
    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error>{
        Sink::start_send(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, item)
    }
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>{
        Sink::poll_ready(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
    fn poll_flush(
        self: Pin<&mut Self>, 
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>>{
        Sink::poll_flush(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
    fn poll_close(
        self: Pin<&mut Self>, 
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>>{
        Sink::poll_close(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
}

pub struct ConnectionRecvPart{
    stream: Pin<Box<dyn Stream<Item=Result<model::GatewayEvent,Error>> + Send + 'static>>,
    heartbeat_timer: tokio::timer::Interval,
}

pub struct ConnectionSendPart{
    sink: CloseOnDrop<Box<dyn Sink<model::GatewayCommand,Error=Error> + Send + Unpin + 'static>,model::GatewayCommand>,
    token: String,
}


pub struct Connection{
    recv: ConnectionRecvPart,
    send: ConnectionSendPart,
}

impl<I,S> Drop for CloseOnDrop<S,I>
    where S: Sink<I> + Unpin
{
    fn drop(&mut self){
        //we ignore this since we can't do anything about it if it fails, and we're only sending the close signal to be courteous
        let _ = self.0.close();
    }
}

///Indicates that you must send a heartbeat
pub struct SendHeartBeat;

impl ConnectionRecvPart{
    pub fn event_stream(self) -> impl Stream<Item=Result<futures::future::Either<SendHeartBeat,model::ReceivableEvent>,Error>>
    {
        use futures::{future::Either,stream::StreamExt};
        let ConnectionRecvPart{heartbeat_timer, stream} = self;
        futures::stream::select(heartbeat_timer.map(Either::Left),futures::StreamExt::map(stream,Either::Right))
            .filter_map(|heartbeat_or_event|{
                futures::future::ready(
                match heartbeat_or_event{
                    Either::Left(_heartbeat) => {
                        Some(Ok(Either::Left(SendHeartBeat)))
                    }
                    Either::Right(Ok(payload)) => {
                        trace!("got payload: {:?}",payload);
                        match payload{
                            model::GatewayEvent::HeartbeatAck => {
                                //don't really care
                                None
                            }
                            model::GatewayEvent::HeartbeatRequest => {
                                Some(Ok(Either::Left(SendHeartBeat)))
                            }
                            model::GatewayEvent::Hello(hello) => {
                                warn!("unexpected hello payload: {:?}",hello);
                                //most resilient thing to do here is just continue probably
                                None
                            }
                            model::GatewayEvent::ReceivableEvent(event) => {
                                Some(Ok(Either::Right(event)))
                            }
                            model::GatewayEvent::Reconnect => {
                                todo!("implement gateway reconnect")
                            }
                            model::GatewayEvent::InvalidSession(_) => {
                                todo!("implement gateway reconnect")
                            }
                        }
                    },
                    Either::Right(Err(e)) => {
                        Some(Err(e))
                    }
                }
                )
            })
    }
}

impl Connection{
    pub async fn run<E,F,Fut>(self, mut f: F) -> Result<(),E>
        where F: FnMut(model::ReceivableEvent,crate::rest_client::Client) -> Fut,
            Fut: std::future::Future<Output = Result<(),E>> + Send + 'static,
            E: std::fmt::Debug + From<Error>
    {
        use futures::future::Either;

        let Connection{send: ConnectionSendPart{mut sink,token},recv} = self;

        let client = crate::rest_client::Client::new(token);

        let mut event_stream = recv.event_stream();
        while let Some(res) = event_stream.next().await{
            match res?{
                Either::Left(_send_heatbeat) => {
                    debug!("sending heartbeat");
                    //TODO: store & send sequence numbers
                    sink.send(crate::model::Heartbeat{last_seq: None}.into()).await?;
                }
                Either::Right(event) => {
                    let fut = f(event,client.clone());
                    tokio::spawn(async{
                        if let Err(ref e) = fut.await{
                            //warn on errors, but expect them to be recoverable, so don't abort
                            warn!("event handler error {:?}",e)
                        }
                    });
                }
            }
        }
        Ok(())
    }

    pub async fn connect<S: Into<String>>(token: S) -> Result<Self,Error>{
        let token = token.into();
        let client = crate::rest_client::Client::new(token.clone());
        let url = client.get_gateway(crate::GATEWAY_VERSION).await?;
        let (stream,_res) = tokio_tungstenite::connect_async(url).compat().await?;
        let (sink,stream) = stream.split();
        let (sink,stream) = (sink.sink_compat(),stream.compat());
        let mut sink: Box<dyn Sink<_,Error=_>+Send+Unpin> = Box::new(sink.sink_map_err(Error::from).with(|payload: model::GatewayCommand|{
            future::lazy(|_|{
                let payload = model::Payload::try_from_command(payload)?;
                let payload = serde_json::to_string(&payload)?;
                trace!("sending payload: {:?}",payload);
                Ok(tungstenite::Message::Text(payload))
            })
        }));
        let mut stream = Box::pin(stream.map_err(Error::from).and_then(|message|{
            future::lazy(|_|{
                let text = message.into_text()?;
                trace!("Parsing: {}",&text);
                let payload: model::Payload = serde_json::from_str(text.as_str())?;
                //TODO: do something with sequence number here
                Ok(payload.received_event_data()?)
            })
        }));

        let payload = stream.try_next().await?;

        debug!("packet, should be hello: {:#?}",payload);
        let hello = payload.unwrap().expect_hello();
        let heartbeat_interval = hello.heartbeat_interval;
        trace!("{:#?}",hello);
        let identify = model::Identify::new(token.clone());
        debug!("sending identify payload: {:#?}",identify);

        sink.send(identify.into()).await?;

        let sink = CloseOnDrop::new(sink);

        let payload = stream.try_next().await?;

        debug!("packet, should be event ready: {:?}",payload);
        let ready = payload.unwrap().expect_event().expect_ready();
        trace!("{:#?}",ready);

        Ok(Self{
            send: ConnectionSendPart{
                sink,token
            },
            recv: ConnectionRecvPart{
                stream,heartbeat_timer: tokio::timer::Interval::new(Instant::now(),Duration::from_millis(heartbeat_interval))
            }
        })
    }
}