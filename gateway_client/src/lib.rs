#![feature(generators,await_macro, async_await, todo_macro)]
extern crate tokio;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate bitflags;
#[macro_use]
extern crate log;

pub use discord_next_model as model;
pub use discord_next_rest as rest_client;

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
    HeartbeatTimer(#[cause] tokio::timer::Error),
    #[fail(display = "An error with a rest operation: {}",_0)]
    RestError(#[cause] discord_next_rest::Error),
}

impl Error{
    pub fn is_recoverable(&self) -> bool{
        match self{
            Error::Ws(_) | Error::HeartbeatTimer(_) => false,
            _other => true,
        }
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
