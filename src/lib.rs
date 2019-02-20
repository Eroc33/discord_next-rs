#![feature(proc_macro,try_from,generators,await_macro, async_await, futures_api)]
extern crate reqwest;
#[macro_use]
extern crate tokio;
extern crate tungstenite;
extern crate tokio_tungstenite;
extern crate tokio_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate url;
extern crate futures;
#[macro_use]
extern crate bitflags;

pub mod model;
mod connection;
mod client;
pub use connection::*;
pub use client::*;

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
}

impl Error{
    pub fn is_recoverable(&self) -> bool{
        match self{
            Error::Ws(_) | Error::HeartbeatTimer(_) => false,
            _ohter => true,
        }
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

impl From<tokio::timer::Error> for Error{
    fn from(e: tokio::timer::Error) -> Self{
        Error::HeartbeatTimer(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_gets_the_gateway() {
        assert!(get_gateway().is_ok());
    }
}
