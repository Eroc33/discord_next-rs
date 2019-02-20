
use url::Url;
use futures::prelude::*;
use crate::Error;
use crate::model;
use std::time::{Duration,Instant};

pub struct CloseOnDrop<S: Sink>(S);

impl<S> Sink for CloseOnDrop<S>
    where S: Sink
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
    fn start_send(
        &mut self, 
        item: Self::SinkItem
    ) -> StartSend<Self::SinkItem, Self::SinkError>{
        self.0.start_send(item)
    }
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError>{
        self.0.poll_complete()
    }
}

pub struct ConnectionRecvPart{
    stream: Box<Stream<Item=model::ReceivablePayload,Error=Error> + Send + 'static>,
    heartbeat_timer: tokio::timer::Interval,
}

pub struct ConnectionSendPart{
    sink: CloseOnDrop<Box<Sink<SinkItem=model::SendablePayload,SinkError=Error> + Send + 'static>>,
    token: String,
}


pub struct Connection{
    recv: ConnectionRecvPart,
    send: ConnectionSendPart,
}

impl<S> Drop for CloseOnDrop<S>
    where S: Sink
{
    fn drop(&mut self){
        //we ignore this since we can't do anything about it if it fails, and we're only sending the close signal to be courteous
        let _ = self.0.close();
    }
}

///Indicates that you must send a heartbeat
pub struct SendHeartBeat;

impl ConnectionRecvPart{
    pub fn event_stream(self) -> impl Stream<Item=futures::future::Either<SendHeartBeat,model::ReceivableEvent>,Error=Error>
    {
        use futures::future::Either;
        let ConnectionRecvPart{mut heartbeat_timer, mut stream} = self;
        heartbeat_timer.map_err(Error::from).map(Either::A).select(stream.map(Either::B)).filter_map(|heartbeat_or_event|{
            match heartbeat_or_event{
                Either::A(heartbeat) => {
                    Some(Either::A(SendHeartBeat))
                }
                Either::B(payload) => {
                    eprintln!("got payload: {:?}",payload);
                    match payload{
                        model::ReceivablePayload::HeartbeatAck => {
                            //don't really care
                            None
                        }
                        model::ReceivablePayload::HeartbeatRequest => {
                            eprintln!("sending heartbeat");
                            Some(Either::A(SendHeartBeat))
                        }
                        model::ReceivablePayload::Hello(hello) => {
                            eprint!("WARNING: unexpected hello payload: {:?}",hello);
                            //most resilient thing to do here is just continue probably
                            None
                        }
                        model::ReceivablePayload::ReceivableEvent(event) => {
                            Some(Either::B(event))
                        }
                    }
                }
            }
        })
    }
}

impl Connection{
    pub async fn run<F,Fut>(self, mut f: F) -> Result<(),Error>
        where F: FnMut(model::ReceivableEvent,crate::Client) -> Fut,
            Fut: std::future::Future<Output = Result<(),Error>> + Send + 'static
    {
        use futures::future::Either;
        use tokio::prelude::*;

        let Connection{send: ConnectionSendPart{mut sink,token},recv} = self;

        let mut client = crate::Client::new(token);

        let mut event_stream = recv.event_stream();
        while let Some(res) = await!(event_stream.next()){
            match res?{
                Either::A(_send_heatbeat) => {
                    eprintln!("sending heartbeat");
                    sink = await!(sink.send(model::Heartbeat.into()))?;
                }
                Either::B(event) => {
                    let fut = f(event,client.clone());
                    tokio::spawn_async(async{
                        if let Err(ref e) =  await!(fut){
                            //warn on errors, but expect them to be recoverable, so don't abort
                            eprintln!("WARNING: error {:?}",e)
                        }
                    });
                }
            }
        }
        Ok(())
    }
}


const API_BASE: &str = " https://discordapp.com/api/v6";

const GATEWAY_VERSION: u8 = 6;

pub fn get_gateway() -> Result<Url,Error>{
    #[derive(Deserialize)]
    struct GatewayResponse{
        url: String
    }
    let res: GatewayResponse = reqwest::get(&(API_BASE.to_owned()+"/gateway"))?.json()?;
    Ok(Url::parse(format!("{}?v={}&encoding=json",res.url,GATEWAY_VERSION).as_str())?)
}

pub async fn connect_to_gateway(token: String) -> Result<Connection,Error>{
    let url = get_gateway()?;
    let (stream,_res) = await!(tokio_tungstenite::connect_async(url))?;
    let (sink,stream) = stream.split();
    let sink: Box<Sink<SinkItem=_,SinkError=_>+Send> = Box::new(sink.sink_map_err(Error::from).with(|payload: model::SendablePayload|{
        let payload = model::Payload::try_from_sendable(payload)?;
        let payload = serde_json::to_string(&payload)?;
        eprintln!("sending payload: {:?}",payload);
        Ok(tungstenite::Message::Text(payload))
    }));
    let stream = Box::new(stream.map_err(Error::from).and_then(|message|{
        let text = message.into_text()?;
        eprintln!("Parsing: {}",&text);
        let payload: model::Payload = serde_json::from_str(&text)?;
        Ok(payload.received_event_data()?)
    }));
    let (payload,stream) = await!(stream.into_future().map_err(|(err,_strm)| err))?;

    eprintln!("packet, should be hello: {:#?}",payload);
    let hello = payload.unwrap().expect_hello();
    let heartbeat_interval = hello.heartbeat_interval;
    eprintln!("{:#?}",hello);
    let identify = model::Identify::new(token.clone());
    eprintln!("sending identify payload: {:#?}",identify);

    let sink = CloseOnDrop(await!(sink.send(identify.into()))?);

    let (payload,stream) = await!(stream.into_future().map_err(|(err,_strm)| err))?;

    eprintln!("packet, should be event ready: {:?}",payload);
    let ready = payload.unwrap().expect_event().expect_ready();
    eprintln!("{:?}",ready);

    Ok(Connection{
        send: ConnectionSendPart{
            sink,token
        },
        recv: ConnectionRecvPart{
            stream,heartbeat_timer: tokio::timer::Interval::new(Instant::now(),Duration::from_millis(heartbeat_interval))
        }
    })
}