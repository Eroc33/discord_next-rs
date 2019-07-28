
use crate::{
    model,
    extensions::SinkExt as _,
};
use std::{
    pin::Pin,
    convert::{TryFrom,TryInto},
    time::{Duration,Instant},
};
use futures::{
    sink::{Sink},
    stream,
    prelude::*,
    compat::*,
    channel::mpsc::{UnboundedReceiver,UnboundedSender},
    select,
};
use url::Url;
use rust_sodium::crypto::secretbox;
use tracing::*;
use futures_01::stream::Stream as _;
use model::voice::udp::RTP_HEADER_LEN;
use tracing_futures::Instrument as _;

pub mod ffmpeg;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An error occured while parsing a url {:?}",_0)]
    Url(#[cause] url::ParseError),
    #[fail(display = "An error occured during a websocket operation {:?}",_0)]
    Ws(#[cause] tungstenite::error::Error),
    #[fail(display = "An error occured during (de)serialization {:?}",_0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "An error with a timer operation for heartbeat {:?}",_0)]
    HeartbeatTimer(#[cause] tokio::timer::Error),
    #[fail(display = "An error with a rest operation: {}",_0)]
    RestError(#[cause] discord_next_rest::Error),
    #[fail(display = "Gateway connection closed: {:?}",_0)]
    ConnectionClosed(Option<model::CloseCode>),
    #[cfg(feature="voice")]
    #[fail(display = "Voice connection closed: {:?}",_0)]
    VoiceConnectionClosed(Option<model::voice::CloseCode>),
    #[fail(display = "Couldn't send on gateway connection. It is most likely closed: {:?}",_0)]
    SendError(#[cause] futures::channel::mpsc::SendError),
    #[fail(display = "IO error: {:?}",_0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "Ip discovery failed: {:?}",_0)]
    IpDiscovery(#[cause] model::voice::udp::DiscoveryPacketError),
    #[fail(display = "FromPayloadError: {:?}",_0)]
    FromPayload(#[cause] model::FromPayloadError),
}

impl From<model::FromPayloadError> for Error{
    fn from(e: model::FromPayloadError) -> Self{
        Error::FromPayload(e)
    }
}

impl From<model::voice::udp::DiscoveryPacketError> for Error{
    fn from(e: model::voice::udp::DiscoveryPacketError) -> Self{
        Error::IpDiscovery(e)
    }
}

impl From<std::io::Error> for Error{
    fn from(e: std::io::Error) -> Self{
        Error::Io(e)
    }
}

impl From<futures::channel::mpsc::SendError> for Error{
    fn from(e: futures::channel::mpsc::SendError) -> Self{
        Error::SendError(e)
    }
}

impl From<discord_next_rest::Error> for Error{
    fn from(e: discord_next_rest::Error) -> Self{
        Error::RestError(e)
    }
}

impl From<url::ParseError> for Error{
    fn from(e: url::ParseError) -> Self{
        Error::Url(e)
    }
}

impl From<tungstenite::error::Error> for Error{
    fn from(e: tungstenite::error::Error) -> Self{
        Error::Ws(e)
    }
}

impl From<serde_json::Error> for Error{
    fn from(e: serde_json::Error) -> Self{
        Error::Json(e)
    }
}

#[cfg(feature="connection")]
impl From<tokio::timer::Error> for Error{
    fn from(e: tokio::timer::Error) -> Self{
        Error::HeartbeatTimer(e)
    }
}

pub trait AudioStream{
    fn read_frame(&mut self, buffer: &mut [i16]);
}

impl AudioStream for Box<dyn AudioStream + Send>{
    fn read_frame(&mut self, buffer: &mut [i16]){
        self.as_mut().read_frame(buffer)
    }
}

struct ConnectionAudioRunner{
    sink: UnboundedSender<model::voice::VoiceCommand>,
    audio_encoder: opus::Encoder,
    secret_key: secretbox::Key,
    seq_num: u16,
    timestamp: u32,
    ssrc: u32,
    udp_addr: std::net::SocketAddr,
    udp: tokio::net::UdpSocket,
    speaking: bool,
}

impl ConnectionAudioRunner{
    async fn set_speaking(&mut self, speaking: bool) -> Result<(),Error>
    {
        if self.speaking == speaking{
            return Ok(());
        }
        self.speaking = speaking;
        trace!("setting speaking status to: {}",speaking);
        self.sink.send(model::voice::SetSpeaking{
            speaking: speaking,
            delay: 0,
            ssrc: self.ssrc,
        }.into()).await?;
        trace!("speaking status set to: {}", speaking);
        Ok(())
    }
    pub async fn run(mut self, mut audio_stream: impl AudioStream) -> Result<(),Error>
    {
        const ENCRYPTION_HEADROOM: usize = 16;
        const FRAME_DURATION: Duration = Duration::from_millis(20);
        let mut udp_timer = tokio::timer::Interval::new_interval(FRAME_DURATION);
        loop{
            //let frame_start = Instant::now();
            self.set_speaking(true).await?;
            //let mut packet_buf = [0u8;512];
            //self.udp.recv_from(&mut packet_buf).await?;

            let mut packet_buf = [0u8;512];
            //TODO: allow switching to stereo (960*2 samples)
            let mut audio_buf = [0i16;960];

            audio_stream.read_frame(&mut audio_buf);

            //create packet
            let packet_len = {
                let (header,body) = packet_buf.split_at_mut(RTP_HEADER_LEN);

                model::voice::udp::rtp_header(header, self.seq_num, self.timestamp, self.ssrc)?;

                let nonce = secretbox::Nonce(model::voice::udp::nonce(&header));

                let extent = body.len()-ENCRYPTION_HEADROOM;
                let audio_len = self.audio_encoder.encode(&audio_buf[..],&mut body[..extent]).expect("FIXME");

                let encrypted = secretbox::seal(&body[..audio_len], &nonce, &self.secret_key);

                body[..encrypted.len()].copy_from_slice(&encrypted);
                RTP_HEADER_LEN+encrypted.len()
            };

            self.seq_num = self.seq_num.wrapping_add(1);
		    self.timestamp = self.timestamp.wrapping_add(960);

            udp_timer.next().await;

            self.udp.send_to(&packet_buf[..packet_len],&self.udp_addr).await?;

            // let elapsed = Instant::now() - frame_start;
            // trace!("audio packet sent in {:?}", elapsed);

            // if elapsed < FRAME_DURATION {
            //     let deadline = frame_start + FRAME_DURATION;
            //     trace!("waiting {:?} for frame duration to elapse",deadline.checked_duration_since(Instant::now()));
            //     udp_timer.next().await;
            // }
        }
    }
}

struct ConnectionWebsocketRunner{
    sink: UnboundedSender<model::voice::VoiceCommand>,
    stream: stream::Fuse<Pin<Box<dyn Stream<Item=Result<model::Payload,Error>> + Send +'static>>>,
    heartbeat_timer: stream::Fuse<tokio::timer::Interval>,
}

impl ConnectionWebsocketRunner{
    async fn turn(&mut self) -> Result<bool,Error>{
        select!{
            _beat = self.heartbeat_timer.next() => {
                self.sink.send(crate::model::voice::VoiceCommand::Heartbeat(0)).await?;
                return Ok(false);
            },
            payload = self.stream.next() => {
                let payload = match payload{
                    None => return Ok(false),
                    Some(Err(e)) => return Err(e),
                    Some(Ok(payload)) => payload,
                };
                trace!("got voice payload: {:?}",payload);
                let voice_event: model::voice::VoiceEvent = match payload.try_into(){
                    Ok(o) => o,
                    Err(model::FromPayloadError::UnknownOpcode(op)) => {
                        warn!("Unknown voice opcode {}", op);
                        return Ok(false);
                    },
                    Err(other) => {
                        return Err(other.into());
                    }
                };
                trace!("got voice_event: {:?}",voice_event);
                match voice_event{
                    model::voice::VoiceEvent::HeartbeatACK => {
                        //don't really care
                        trace!("HeartbeatACK");
                    }
                    model::voice::VoiceEvent::Hello(hello) => {
                        warn!("unexpected hello payload: {:?}",hello);
                        //most resilient thing to do here is just continue probably
                    }
                    model::voice::VoiceEvent::Ready(ready) => {
                        warn!("VoiceEvent::Ready not implemented")
                    }
                    model::voice::VoiceEvent::Resumed => {
                        warn!("VoiceEvent::Resumed not implemented")
                    }
                    model::voice::VoiceEvent::SessionDescription(session_description) => {
                        warn!("VoiceEvent::SessionDescription not implemented")
                    }
                    model::voice::VoiceEvent::Speaking(speaking) => {
                        warn!("VoiceEvent::Speaking not implemented")
                    }
                    model::voice::VoiceEvent::ClientDisconnect => {
                        warn!("VoiceEvent::ClientDisconnect not implemented")
                    }
                }
                return Ok(false);
            },
            complete => return Ok(true),
            default => return Ok(false),
        }
    }

    async fn run(mut self) -> Result<(),Error>{
        while !self.turn().await?{}
        Ok(())
    }
}

pub struct Connection{
    sink: UnboundedSender<model::voice::VoiceCommand>,
    stream: Pin<Box<dyn Stream<Item=Result<model::Payload,Error>> + Send +'static>>,
    audio_encoder: opus::Encoder,
    secret_key: secretbox::Key,
    seq_num: u16,
    timestamp: u32,
    ssrc: u32,
    udp_addr: std::net::SocketAddr,
    udp: tokio::net::UdpSocket,
    heartbeat_timer: tokio::timer::Interval,
    speaking: bool,
}

impl Connection{
    fn decompose(self) -> (ConnectionAudioRunner,ConnectionWebsocketRunner)
    {
        let Connection{sink, stream, audio_encoder, secret_key, seq_num, timestamp, ssrc, udp_addr, udp, heartbeat_timer, speaking} = self;
        (ConnectionAudioRunner{
            sink: sink.clone(), audio_encoder, secret_key, seq_num, timestamp, ssrc, udp_addr, udp, speaking,
        },ConnectionWebsocketRunner{
            sink, stream: stream.fuse(), heartbeat_timer: heartbeat_timer.fuse(),
        })
    }

    pub fn connect(gateway_connection: &mut crate::connection::Connection, guild_id: model::GuildId, channel_id: Option<model::ChannelId>) -> impl Future<Output=Result<Self,Error>>
    {
        let sender = gateway_connection.clone_writer();
        let vsu = gateway_connection.voice_server_updates_for(guild_id);
        let user_id = gateway_connection.user.id;
        let session_id = gateway_connection.session_id.clone();
        Self::connect_internal(sender,vsu,user_id, session_id, guild_id,channel_id)
    }
    async fn connect_internal(mut sender: crate::connection::ConnectionWriter, mut vsu: UnboundedReceiver<crate::connection::VoiceInfo>, user_id: model::UserId, session_id: String, guild_id: model::GuildId, channel_id: Option<model::ChannelId>) -> Result<Self,Error>{

        trace!("sending voice state update");
        sender.sink.send(model::VoiceStateUpdate{
            guild_id,
            channel_id,
            self_deaf: false,
            self_mute: false,
        }.into()).await?;

        //TODO: add a timeout on this, as we'll never get the VoiceServerUpdate if the room we try to join is full
        trace!("awaiting new voice info");
        let voice_info: crate::connection::VoiceInfo = vsu.next().await.expect("gateway dropped while constructing voice::Connection");
        trace!("got new voice info");

        let url = Url::parse(&format!("wss://{}?v=3",voice_info.endpoint.trim_end_matches(":80")))?;

        trace!("connecting voice websocket: {}", url);
        let (stream,_res) = tokio_tungstenite::connect_async(url).compat().await?;
        let (sink,stream) = stream.split();
        let (sink,stream) = (sink.sink_compat(),stream.compat());
        let mut sink: Pin<Box<dyn Sink<model::voice::VoiceCommand,Error=Error>+Send+Unpin>> = Box::pin(sink.sink_map_err(Error::from).with(|payload: model::voice::VoiceCommand|{
            future::lazy(|_|{
                let payload = model::Payload::try_from_voice_command(payload)?;
                let payload = serde_json::to_string(&payload)?;
                trace!("sending payload: {:?}",payload);
                Ok(tungstenite::Message::Text(payload))
            })
        }));
        let mut stream: Pin<Box<dyn Stream<Item=Result<model::Payload,Error>>+Send+Unpin>> = Box::pin(stream.map_err(Error::from).and_then(|message|{
            future::lazy(|_|{
                if let tungstenite::Message::Close(close_frame) = message {
                    return Err(Error::VoiceConnectionClosed(close_frame.and_then(|frame| model::voice::CloseCode::try_from(Into::<u16>::into(frame.code)).ok())));
                }
                let text = message.into_text()?;
                trace!("Parsing: {}",&text);
                let payload: model::Payload = serde_json::from_str(text.as_str())?;
                Ok(payload)
            })
        }));

        trace!("awaiting voice packet");
        let event: model::voice::VoiceEvent = stream.try_next().await?.unwrap().try_into()?;

        debug!("packet, should be hello: {:#?}",event);
        let hello = event.expect_hello();

        trace!("sending voice identify");
        sink.send(model::voice::Identify{
            server_id: guild_id,
            user_id,
            session_id,
            token: voice_info.token,
        }.into()).await?;

        trace!("awaiting voice packet");
        let event: model::voice::VoiceEvent = stream.try_next().await?.unwrap().try_into()?;

        debug!("packet, should be ready: {:#?}",event);
        let ready = event.expect_ready();

        let udp_addr = std::net::SocketAddr::new(ready.ip, ready.port);

        eprintln!("udp endpoint: {:?}", udp_addr);

        let mut udp = tokio::net::UdpSocket::bind(&std::net::SocketAddr::new(std::net::Ipv4Addr::new(0, 0, 0, 0).into(),0))?;

        {
            trace!("sending ip discovery packet");
            //send ip discovery packet
            let buf = model::voice::udp::discovery_request([0u8;70], ready.ssrc)?;
            udp.send_to(&buf,&udp_addr).await?;
            trace!("sent ip discovery packet");
        }
        let (ip,port) = {
            trace!("recieving ip discovery packet");
            //recieve ip discovery response
            let mut buf = [0u8;70];
            let (len, _peer_addr) = udp.recv_from(&mut buf).await?;

            debug!("discovery_packet: {:#X?}", &buf[..len]);
            model::voice::udp::parse_discovery_response(buf)
        }?;

        sink.send(model::voice::SelectProtocol{
            protocol: "udp".into(),
            data: serde_json::to_value(model::voice::UdpProtocolData{
                address: ip,
                port,
                mode: "xsalsa20_poly1305".into(),
            })?
        }.into()).await?;
        
        let session_description = loop{
            trace!("awaiting voice packet");
            let event: model::voice::VoiceEvent = stream.try_next().await?.unwrap().try_into()?;

            debug!("packet, should be session_description: {:#?}",event);
            if let model::voice::VoiceEvent::SessionDescription(session_description) = event{
                break session_description;
            }
        };

        Ok(Connection{
            sink: sink.unbounded_channeled(),
            stream,
            audio_encoder: opus::Encoder::new(model::voice::udp::SAMPLE_RATE, opus::Channels::Mono, opus::Application::Audio).expect("Couldn't create opus encoder"),
            secret_key: secretbox::Key::from_slice(&session_description.secret_key).expect("key size for xsalsa20poly1305 should be 32"),
            ssrc: ready.ssrc,
            seq_num: 0,
            timestamp: 0,
            udp_addr,
            udp,
            heartbeat_timer: tokio::timer::Interval::new(Instant::now(), Duration::from_millis((hello.heartbeat_interval * 3)/4)),
            speaking: false,
        })
    }

    pub async fn run(self, audio_stream: impl AudioStream){
        let (audio_runner,ws_runner) = self.decompose();

        tokio::spawn(ws_runner.run().map(|res|{
            res.expect("FIXME");
        }).instrument(span!(Level::INFO, "ws_runner")));

        audio_runner.run(audio_stream).map(|res|{
            res.expect("FIXME");
        }).instrument(span!(Level::INFO, "audio_runner")).await;
    }
}