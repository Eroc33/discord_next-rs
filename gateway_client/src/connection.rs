
use std::{
    time::{Duration,Instant},
    pin::Pin,
    convert::{TryFrom,TryInto},
};
#[cfg(feature="voice")]
use std::collections::HashMap;
use futures::{
    prelude::*,
    sink::{Sink},
    stream,
    compat::*,
    channel::mpsc::{UnboundedSender,UnboundedReceiver},
};
use crate::{
    extensions::*,
    close_on_drop::CloseOnDrop,
    Error,
    model,
};
use tracing::*;
use futures_01::stream::Stream as _;

pub struct VoiceInfo{
    pub token: String,
    pub endpoint: String,
}

#[derive(Clone)]
pub (crate) struct ConnectionWriter{
    pub sink: UnboundedSender<model::GatewayCommand>,
}

pub struct Connection{
    pub session_id: String,
    stream: stream::Fuse<Pin<Box<dyn Stream<Item=Result<model::Payload,Error>> + Send + 'static>>>,
    heartbeat_timer: stream::Fuse<tokio::timer::Interval>,
    sink: UnboundedSender<model::GatewayCommand>,
    token: String,
    seq_num: Option<u64>,
    #[cfg(feature="voice")]
    voice_update_store: VoiceStateStore,
    pub user: model::User,
}

impl Connection{
    pub (crate) fn clone_writer(&self) -> ConnectionWriter
    {
        ConnectionWriter{
            sink: self.sink.clone()
        }
    }

    fn update_seq_num(&mut self, new_seq_num: Option<u64>){
        //TODO: should we only count upwards?
        self.seq_num = new_seq_num;
    }

    #[cfg(feature="voice")]
    pub (crate) fn voice_update_store(&self) -> &VoiceStateStore{
        &self.voice_update_store
    }
    
    #[cfg(feature="voice")]
    async fn update_voice_info(&mut self, voice_server_update: model::VoiceServerUpdate) -> Result<(),VoiceStateUpdateError>{
        self.voice_update_store.send_event(&voice_server_update.guild_id,VoiceInfo{
            endpoint: voice_server_update.endpoint,
            token: voice_server_update.token,
        }).await
    }

    async fn handle_dispatch<E,F,Fut>(&mut self, event: model::ReceivableEvent, client: &crate::rest_client::Client, f: &mut F) -> Result<(),Error>
        where F: FnMut(&mut Self, model::ReceivableEvent,crate::rest_client::Client) -> Fut,
            Fut: std::future::Future<Output = Result<(),E>> + Send + 'static,
            E: std::fmt::Debug + From<Error>
    {
        match event{
            #[cfg(feature="voice")]
            model::ReceivableEvent::VoiceServerUpdate(voice_server_update) => {
                match self.update_voice_info(voice_server_update).await{
                    Ok(_) => {},
                    Err(_e) => {
                        warn!("Send error when updating voice state.")
                    }
                }
            },
            other => {
                let fut = f(self, other,client.clone());
                tokio::spawn(async{
                    if let Err(ref e) = fut.await{
                        //warn on errors, but expect them to be recoverable, so don't abort
                        warn!("event handler error {:?}",e)
                    }
                });
            }
        }
        Ok(())
    }

    ///returns true if complete
    pub async fn turn<E,F,Fut>(&mut self, client: &crate::rest_client::Client, f: &mut F) -> Result<bool,Error>
        where F: FnMut(&mut Self, model::ReceivableEvent,crate::rest_client::Client) -> Fut,
            Fut: std::future::Future<Output = Result<(),E>> + Send + 'static,
            E: std::fmt::Debug + From<Error>
    {
        use futures::select;

        select!{
            _beat = self.heartbeat_timer.next() => {
                self.sink.send(crate::model::Heartbeat{last_seq: None}.into()).await?;
                return Ok(false);
            },
            payload = self.stream.next() => {
                let payload = match payload{
                    None => return Ok(false),
                    Some(Err(e)) => return Err(e),
                    Some(Ok(payload)) => payload,
                };
                self.update_seq_num(payload.s);
                let gateway_event: model::GatewayEvent = match payload.try_into(){
                    Ok(o) => o,
                    Err(model::FromPayloadError::UnknownOpcode(op)) => {
                        warn!("Unknown voice opcode {}", op);
                        return Ok(false);
                    },
                    Err(other) => {
                        return Err(other.into());
                    }
                };
                trace!("got gateway_event: {:?}",gateway_event);
                match gateway_event{
                    model::GatewayEvent::HeartbeatAck => {
                        //don't really care
                    }
                    model::GatewayEvent::HeartbeatRequest => {
                        self.sink.send(crate::model::Heartbeat{last_seq: None}.into()).await?;
                    }
                    model::GatewayEvent::Hello(hello) => {
                        warn!("unexpected hello payload: {:?}",hello);
                        //most resilient thing to do here is just continue probably
                    }
                    model::GatewayEvent::ReceivableEvent(event) => {
                        self.handle_dispatch(event,client,f).await?;
                    }
                    model::GatewayEvent::Reconnect => {
                        todo!("implement gateway reconnect")
                    }
                    model::GatewayEvent::InvalidSession(_) => {
                        todo!("implement gateway invalid session")
                    }
                }
                return Ok(false);
            },
            complete => return Ok(true),
            default => return Ok(false),
        }
    }

    //runs Connection::turn to completion
    pub async fn run<E,F,Fut>(mut self, mut f: F) -> Result<(),Error>
        where F: FnMut(&mut Self, model::ReceivableEvent,crate::rest_client::Client) -> Fut,
            Fut: std::future::Future<Output = Result<(),E>> + Send + 'static,
            E: std::fmt::Debug + From<Error>
    {
        let client = crate::rest_client::Client::new(self.token.clone());
        while !self.turn(&client, &mut f).await?{}
        Ok(())
    }

    pub async fn connect<S: Into<String>>(token: S) -> Result<Self,Error>{
        let token = token.into();
        let client = crate::rest_client::Client::new(token.clone());
        let url = client.get_gateway(crate::GATEWAY_VERSION).await?;
        let (stream,_res) = tokio_tungstenite::connect_async(url).compat().await?;
        let (sink,stream) = stream.split();
        let (sink,stream) = (sink.sink_compat(),stream.compat());
        let mut sink: Box<dyn Sink<model::GatewayCommand,Error=Error>+Send+Unpin> = Box::new(sink.sink_map_err(Error::from).with(|payload: model::GatewayCommand|{
            future::lazy(|_|{
                let payload = model::Payload::try_from_command(payload)?;
                let payload = serde_json::to_string(&payload)?;
                trace!("sending payload: {:?}",payload);
                Ok(tungstenite::Message::Text(payload))
            })
        }));
        let mut stream = Box::pin(stream.map_err(Error::from).and_then(|message|{
            future::lazy(|_|{
                if let tungstenite::Message::Close(close_frame) = message {
                    return Err(Error::ConnectionClosed(close_frame.and_then(|frame| model::CloseCode::try_from(Into::<u16>::into(frame.code)).ok())));
                }
                let text = message.into_text()?;
                trace!("Parsing: {}",&text);
                let payload: model::Payload = serde_json::from_str(text.as_str())?;
                Ok(payload)
            })
        }));

        let event: model::GatewayEvent = stream.try_next().await?.unwrap().try_into()?;

        debug!("packet, should be hello: {:#?}",event);
        let hello = event.expect_hello();
        let heartbeat_interval = hello.heartbeat_interval;
        trace!("{:#?}",hello);
        let identify = model::Identify::new(token.clone());
        debug!("sending identify payload: {:#?}",identify);

        sink.send(identify.into()).await?;

        let sink = CloseOnDrop::new(sink);

        let event: model::GatewayEvent = stream.try_next().await?.unwrap().try_into()?;

        debug!("packet, should be event ready: {:?}",event);
        let ready = event.expect_event().expect_ready();
        trace!("{:#?}",ready);

        Ok(Self{
            session_id: ready.session_id,
            sink: sink.unbounded_channeled(),
            token,
            user: ready.user,
            stream: (stream as Pin<Box<dyn Stream<Item=Result<model::Payload,Error>> + Send + 'static>>).fuse(),
            heartbeat_timer: tokio::timer::Interval::new(Instant::now(),Duration::from_millis(heartbeat_interval)).fuse(),
            seq_num: None,
            #[cfg(feature="voice")]
            voice_update_store: Default::default(),
        })
    }
}

pub type VoiceStateStore = EventRouter<model::GuildId,VoiceInfo>;

use std::sync::Arc;
use futures::lock::Mutex;
use std::hash::Hash;

pub struct VoiceStateUpdateError;

struct EventRouterInner<K: Eq + Hash,V>{
    routes: Mutex<HashMap<K,UnboundedSender<V>>>,
}

//#[derive(Default)] doesn't work (see https://github.com/rust-lang/rust/issues/26925) so we implement it manually
impl<K: Eq + Hash,V> Default for EventRouterInner<K,V>{
    fn default() -> Self{
        Self{
            routes: Mutex::new(Default::default())
        }
    }
}

impl<K,V> EventRouterInner<K,V>
    where K: Eq + Hash + std::fmt::Debug
{
    pub fn register(&self, key: K) -> UnboundedReceiver<V>{
        loop{
            if let Some(mut locked) = self.routes.try_lock(){
                let (tx,rx) = futures::channel::mpsc::unbounded();
                locked.insert(key,tx);
                return rx;
            }
        }
    }
    pub async fn send_event(&self, key: &K, event: V) -> Result<(),VoiceStateUpdateError>{
        if let Some(mut chan) = self.routes.lock().await.get(key){
            info!("Voice update for {:?}",key);
            //TODO: remove channels where send fails? (definitely want to do this if the failure is due to disconnection)
            chan.send(event).await.map_err(|_| VoiceStateUpdateError)?;
        }else{
            info!("Unrouted event");
        }
        Ok(())
    }
}

pub struct EventRouter<K: Eq + Hash,V>{
    inner: Arc<EventRouterInner<K,V>>,
}

//#[derive(Clone)] doesn't work (see https://github.com/rust-lang/rust/issues/26925) so we implement it manually
impl<K: Eq + Hash,V> Clone for EventRouter<K,V>{
    fn clone(&self) -> Self{
        Self{
            inner: self.inner.clone()
        }
    }
}

//#[derive(Default)] doesn't work (see https://github.com/rust-lang/rust/issues/26925) so we implement it manually
impl<K: Eq + Hash,V> Default for EventRouter<K,V>{
    fn default() -> Self{
        Self{
            inner: Arc::new(Default::default())
        }
    }
}

impl<K,V> EventRouter<K,V>
    where K: Eq + Hash + std::fmt::Debug
{
    pub fn register(&self, key: K) -> UnboundedReceiver<V>{
        self.inner.register(key)
    }
    pub async fn send_event(&self, key: &K, event: V) -> Result<(),VoiceStateUpdateError>{
        self.inner.send_event(key,event).await
    }
}