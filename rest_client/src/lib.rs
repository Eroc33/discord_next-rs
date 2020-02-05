#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
use thiserror::Error;

pub use discord_next_model as model;

mod client;
pub use client::*;

pub (crate) const API_BASE: &str = "https://discordapp.com/api/v6";

#[derive(Debug, Error)]
pub enum Error {
    #[error("An http error occurred {0:?}")]
    Http (#[from] reqwest::Error),
    #[error("An error occured while parsing a url {0:?}")]
    Url(#[from] url::ParseError),
    #[error("An error occured during (de)serialization {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("An embed was too big {:?}",_0)]
    EmbedTooBig(#[from] model::EmbedTooBigError),
    #[error("Was rate limited too many times (>={0}) while executing: {1}")]
    TooManyRetries(u16,String),
    #[error("An error with a timer operation for ratelimiting {0:?}")]
    Timer(#[from] tokio::time::Error),
    #[error("A non success response code was returned from an http request: {0:?}")]
    UnsuccessfulHttp(http::StatusCode),
    #[error("An error while building an http data structure {0:?}")]
    HttpBuilderError(#[from] http::Error),
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
