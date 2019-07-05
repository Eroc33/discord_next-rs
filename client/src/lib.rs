#![feature(try_from,generators,await_macro, async_await, futures_api)]
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

pub use discord_next_model as model;

mod connection;
mod client;
pub use connection::*;
pub use client::*;

pub (crate) const API_BASE: &str = " https://discordapp.com/api/v6";
pub (crate) const GATEWAY_VERSION: u8 = 6;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An http error occurred {:?}",_0)]
    Http (#[cause] reqwest::Error),
    #[fail(display = "An error occured while parsing a url {:?}",_0)]
    Url(#[cause] url::ParseError),
    #[fail(display = "An error occured during a websocket operation {:?}",_0)]
    Ws(#[cause] tungstenite::error::Error),
    #[fail(display = "An error occured during (de)serialization {:?}",_0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "An error with a timer operation for heartbeat {:?}",_0)]
    HeartbeatTimer(#[cause] tokio::timer::Error),
    #[fail(display = "An embed was too big {:?}",_0)]
    EmbedTooBig(#[cause] model::EmbedTooBigError),
    #[fail(display = "Was rate limited too many times (>={}) while executing: {}",_0,_1)]
    TooManyRetries(u16,String)
}

impl Error{
    pub fn is_recoverable(&self) -> bool{
        match self{
            Error::Ws(_) | Error::HeartbeatTimer(_) => false,
            _ohter => true,
        }
    }
}

impl From<model::EmbedTooBigError> for Error{
    fn from(e: model::EmbedTooBigError) -> Self{
        Error::EmbedTooBig(e)
    }
}

impl From<reqwest::Error> for Error{
    fn from(e: reqwest::Error) -> Self{
        Error::Http(e)
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
