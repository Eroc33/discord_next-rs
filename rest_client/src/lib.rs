#![feature(generators,await_macro,async_await)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub use discord_next_model as model;

mod client;
pub use client::*;

pub (crate) const API_BASE: &str = "https://discordapp.com/api/v6";

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An http error occurred {:?}",_0)]
    Http (#[cause] hyper::Error),
    #[fail(display = "An error occured while parsing a url {:?}",_0)]
    Url(#[cause] url::ParseError),
    #[fail(display = "An error occured during (de)serialization {:?}",_0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "An embed was too big {:?}",_0)]
    EmbedTooBig(#[cause] model::EmbedTooBigError),
    #[fail(display = "Was rate limited too many times (>={}) while executing: {}",_0,_1)]
    TooManyRetries(u16,String),
    #[fail(display = "An error with a timer operation for ratelimiting {:?}",_0)]
    Timer(#[cause] tokio::timer::Error),
    #[fail(display = "A non success response code was returned from an http request: {:?}",_0)]
    UnsuccessfulHttp(http::StatusCode),
    #[fail(display = "An error while building an http data structure {:?}",_0)]
    HttpBuilderError(#[cause] http::Error),
}
impl From<model::EmbedTooBigError> for Error{
    fn from(e: model::EmbedTooBigError) -> Self{
        Error::EmbedTooBig(e)
    }
}

impl From<http::Error> for Error{
    fn from(e: http::Error) -> Self{
        Error::HttpBuilderError(e)
    }
}

impl From<hyper::Error> for Error{
    fn from(e: hyper::Error) -> Self{
        Error::Http(e)
    }
}

impl From<url::ParseError> for Error{
    fn from(e: url::ParseError) -> Self{
        Error::Url(e)
    }
}

impl From<serde_json::Error> for Error{
    fn from(e: serde_json::Error) -> Self{
        Error::Json(e)
    }
}
impl From<tokio::timer::Error> for Error{
    fn from(e: tokio::timer::Error) -> Self{
        Error::Timer(e)
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
