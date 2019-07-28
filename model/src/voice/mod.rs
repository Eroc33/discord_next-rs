use crate::ids::*;
use std::net::IpAddr;
use crate::Payload;
use serde::{Deserialize,Serialize};
use serde_json as json;
use std::convert::TryFrom;

pub mod udp;

pub mod opcode{
    ///begin a voice websocket connection
    pub const IDENTIFY: u64 = 0;
    ///select the voice protocol
    pub const SELECT_PROTOCOL: u64 = 1;
    ///complete the websocket handshake
    pub const READY: u64 = 2;
    ///keep the websocket connection alive
    pub const HEARTBEAT: u64 = 3;
    ///describe the session
    pub const SESSION_DESCRIPTION: u64 = 4;
    ///indicate which users are speaking
    pub const SPEAKING: u64 = 5;
    ///sent immediately following a received client heartbeat
    pub const HEARTBEAT_ACK: u64 = 6;
    ///resume a connection
    pub const RESUME: u64 = 7;
    ///the continuous interval in milliseconds after which the client should send a heartbeat
    pub const HELLO: u64 = 8;
    ///acknowledge Resume
    pub const RESUMED: u64 = 9;
    ///a client has disconnected from the voice channel
    pub const CLIENT_DISCONNECT: u64 = 13;
}

impl Payload{
    pub fn try_from_voice_command<P: Into<VoiceCommand>>(payload: P) -> Result<Self,serde_json::Error>{
        use self::VoiceCommand::*;
        let payload = payload.into();

        let (op,payload) = match payload{
            Identify(identify) => (opcode::IDENTIFY,serde_json::to_value(identify)?),
            SelectProtocol(select_protocol) => (opcode::SELECT_PROTOCOL,serde_json::to_value(select_protocol)?),
            Heartbeat(nonce) => (opcode::HEARTBEAT,serde_json::to_value(nonce)?),
            SetSpeaking(speaking) => (opcode::SPEAKING,serde_json::to_value(speaking)?),
            Resume(resume) => (opcode::RESUME,serde_json::to_value(resume)?),
        };
        Ok(Payload{
            op,
            d: payload,
            s: None,
            t: None,
        })
    }
}

#[derive(Debug,Deserialize)]
pub enum VoiceEvent{
    ///complete the websocket handshake
    Ready(Ready),
    ///describe the session
    SessionDescription(SessionDescription),
    ///indicate which users are speaking
    Speaking(Speaking),
    ///sent immediately following a received client heartbeat
    //FIXME: heartbeatack doesn't match up with the docs!
    HeartbeatACK,
    ///the continuous interval in milliseconds after which the client should send a heartbeat
    Hello(Hello),
    ///acknowledge Resume
    Resumed,
    ///a client has disconnected from the voice channel
    ClientDisconnect,
}

wrapping_from!(VoiceEvent,Ready,expect_ready);
wrapping_from!(VoiceEvent,SessionDescription,expect_session_description);
wrapping_from!(VoiceEvent,Speaking,expect_speaking);
wrapping_from!(VoiceEvent,Hello,expect_hello);

impl TryFrom<Payload> for VoiceEvent{
    type Error = crate::payload::FromPayloadError;
    fn try_from(payload: Payload) -> Result<Self, Self::Error>
    {
        Ok(match payload.op{
            opcode::READY => json::from_value::<Ready>(payload.d)?.into(),
            opcode::SESSION_DESCRIPTION => json::from_value::<SessionDescription>(payload.d)?.into(),
            opcode::SPEAKING => json::from_value::<Speaking>(payload.d)?.into(),
            //FIXME: heartbeatack doesn't match up with the docs!
            opcode::HEARTBEAT_ACK => VoiceEvent::HeartbeatACK,
            opcode::HELLO => json::from_value::<Hello>(payload.d)?.into(),
            //no data
            opcode::RESUMED => VoiceEvent::Resumed,
            //TODO: what data does this give?
            opcode::CLIENT_DISCONNECT => VoiceEvent::ClientDisconnect,
            other => Err(crate::payload::FromPayloadError::UnknownOpcode(other))?,
        })
    }
}

#[derive(Debug,Serialize)]
pub enum VoiceCommand{
    ///begin a voice websocket connection
    Identify(Identify),
    ///select the voice protocol
    SelectProtocol(SelectProtocol),
    ///keep the websocket connection alive
    Heartbeat(/** nonce */u64),
    ///indicate which users are speaking
    SetSpeaking(SetSpeaking),
    ///resume a connection
    Resume(Resume),
}

wrapping_from!(VoiceCommand,Identify,expect_identify);
wrapping_from!(VoiceCommand,SelectProtocol,expect_select_protocol);
wrapping_from!(VoiceCommand,SetSpeaking,expect_set_speaking);
wrapping_from!(VoiceCommand,Resume,expect_resume);

///begin a voice websocket connection
#[derive(Debug,Serialize)]
pub struct Identify{
    pub server_id: GuildId,
    pub user_id: UserId,
    pub session_id: String,
    pub token: String,
}

///select the voice protocol
#[derive(Debug,Serialize)]
pub struct SelectProtocol{
    pub protocol: String,
    //TODO: determine whether there are fixed fields for "data", or if it changes with protocol. Perhaps use a custom serialization and enum for protocol
    ///See UdpProtocolData
    pub data: json::Value,
}

///complete the websocket handshake
#[derive(Debug,Deserialize)]
pub struct Ready{
    ///Synchronization source identifier
    pub ssrc: u32,
    pub ip: IpAddr,
    pub port: u16,
    ///voice encryption modes
    pub modes: Vec<String>,
    //heartbeat interval *is* sent in this packet, but we are not meant to use it, so it is not deserialized. Instead use heartbeat_interval from Hello on a VoiceConnection
    //heartbeat_interval: u64,
}

///describe the session
#[derive(Debug,Deserialize)]
pub struct SessionDescription{
    pub mode: String,
    pub secret_key: [u8;32]
}

///indicate which users are speaking
#[derive(Debug,Deserialize)]
pub struct Speaking{
    pub speaking: bool,
    pub user_id: UserId,
    pub ssrc: u32,
}

///set whether out user is speaking
#[derive(Debug,Serialize)]
pub struct SetSpeaking{
    pub speaking: bool,
    pub delay: u64,
    pub ssrc: u32,
}

///resume a connection
#[derive(Debug,Serialize)]
pub struct Resume{
    pub server_id: GuildId,
    pub session_id: String,
    pub token: String,
}

///the continuous interval in milliseconds after which the client should send a heartbeat
#[derive(Debug,Deserialize)]
pub struct Hello{
    ///NOTE: There is currently a bug in the Hello payload heartbeat interval. Until it is fixed, please take your heartbeat interval as heartbeat_interval * .75. This warning will be removed and a changelog published when the bug is fixed.
    pub heartbeat_interval: u64,
}

#[derive(Debug,Serialize)]
///data for SelectProtocol when protocol="udp"
pub struct UdpProtocolData{
    pub address: IpAddr,
    pub port: u16,
    pub mode: String,
}


#[derive(Debug)]
pub enum CloseCode{
    ///You sent an invalid opcode.
    /*4001*/UnknownOpcode,
    ///You sent a payload before identifying with the Gateway.
    /*4003*/NotAuthenticated,
    ///The token you sent in your identify payload is incorrect.
    /*4004*/AuthenticationFailed,
    ///You sent more than one identify payload. Stahp.
    /*4005*/AlreadyAuthenticated,
    ///Your session is no longer valid.
    /*4006*/SessionNoLongerValid,
    ///Your session has timed out.
    /*4009*/SessionTimeout,
    ///We can't find the server you're trying to connect to.
    /*4011*/ServerNotFound,
    ///We didn't recognize the protocol you sent.
    /*4012*/UnknownProtocol,
    ///Oh no! You've been disconnected! Try resuming.
    /*4014*/Disconnected,
    ///The server crashed. Our bad! Try resuming.
    /*4015*/VoiceServerCrashed,
    ///We didn't recognize your encryption.
    /*4016*/UnknownEncryptionMode,
}

impl TryFrom<u16> for CloseCode{
    type Error = ();
    fn try_from(close_code: u16) -> Result<Self, Self::Error>
    {
        Ok(match close_code{
            4001 => CloseCode::UnknownOpcode,
            //4002 not documented
            4003 => CloseCode::NotAuthenticated,
            4004 => CloseCode::AuthenticationFailed,
            4005 => CloseCode::AlreadyAuthenticated,
            4006 => CloseCode::SessionNoLongerValid,
            //4007 not documented
            //4008 not documented
            4009 => CloseCode::SessionTimeout,
            //4010 not documented
            4011 => CloseCode::ServerNotFound,
            4012 => CloseCode::UnknownProtocol,
            //4013 not documented
            4014 => CloseCode::Disconnected,
            4015 => CloseCode::VoiceServerCrashed,
            4016 => CloseCode::UnknownEncryptionMode,
            _else => return Err(()),
        })
    }
}