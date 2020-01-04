#![recursion_limit="512"]
extern crate tokio;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate bitflags;

pub use discord_next_model as model;
pub use discord_next_rest as rest_client;

mod close_on_drop;
mod extensions;
#[cfg(feature="voice")]
pub mod voice;
mod connection;
pub use connection::*;

pub (crate) const GATEWAY_VERSION: u8 = 6;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An error occured while parsing a url {:?}",_0)]
    Url(#[cause] url::ParseError),
    #[fail(display = "An error occured during a websocket operation {:?}",_0)]
    Ws(#[cause] tungstenite::error::Error),
    #[fail(display = "An error occured during (de)serialization {:?}",_0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "An error with a timer operation for heartbeat {:?}",_0)]
    HeartbeatTimer(#[cause] tokio::time::Error),
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
    #[fail(display = "FromPayloadError: {:?}",_0)]
    FromPayload(#[cause] model::FromPayloadError),
}

impl Error{
    pub fn is_recoverable(&self) -> bool{
        match self{
            Error::Ws(_) | Error::HeartbeatTimer(_) => false,
            _other => true,
        }
    }
}

impl From<model::FromPayloadError> for Error{
    fn from(e: model::FromPayloadError) -> Self{
        Error::FromPayload(e)
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

impl From<tokio::time::Error> for Error{
    fn from(e: tokio::time::Error) -> Self{
        Error::HeartbeatTimer(e)
    }
}

#[cfg(test)]
mod tests {
    //commented because tokio currently doesn't have a way of getting return values from the runtime.
    //can be re-implemented when https://github.com/tokio-rs/tokio/issues/841 is addressed
    // use super::*;
    // #[test]
    // fn it_gets_the_gateway() {
    //     assert!(get_gateway().is_ok());
    // }
}
