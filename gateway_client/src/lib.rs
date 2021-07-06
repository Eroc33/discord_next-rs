#![recursion_limit = "512"]
extern crate bitflags;
extern crate serde_json;
extern crate tokio;

use thiserror::Error;

pub use discord_next_model as model;
pub use discord_next_rest as rest_client;

mod close_on_drop;
mod connection;
mod extensions;
#[cfg(feature = "voice")]
pub mod voice;
pub use connection::*;

pub(crate) const GATEWAY_VERSION: u8 = 8;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An error occured while parsing a url {0:?}")]
    Url(#[from] url::ParseError),
    #[error("An error occured during a websocket operation {0:?}")]
    Ws(#[from] tungstenite::error::Error),
    #[error("An error occured during (de)serialization {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("An error with a timer operation for heartbeat {0:?}")]
    HeartbeatTimer(#[from] tokio::time::Error),
    #[error("An error with a rest operation: {0}")]
    RestError(#[from] discord_next_rest::Error),
    #[error("Gateway connection closed: {0:?}")]
    ConnectionClosed(Option<model::CloseCode>),
    #[cfg(feature = "voice")]
    #[error("Voice connection closed: {0:?}")]
    VoiceConnectionClosed(Option<model::voice::CloseCode>),
    #[error("Couldn't send on gateway connection. It is most likely closed: {0:?}")]
    SendError(#[from] futures::channel::mpsc::SendError),
    #[error("IO error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("FromPayloadError: {0:?}")]
    FromPayload(#[from] model::FromPayloadError),
    #[error("UserError: {0:?}")]
    Generic(#[from] anyhow::Error),
}

impl Error {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::Ws(_) | Error::HeartbeatTimer(_) => false,
            _other => true,
        }
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
