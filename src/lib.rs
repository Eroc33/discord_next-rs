#![feature(try_from)]
extern crate reqwest;
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
extern crate failure_await as futures;
extern crate url;
extern crate futures;

use url::Url;
use futures::prelude::*;

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::stream::Stream as StreamSwitcher;
use tokio::net::TcpStream;
use tokio_tls::TlsStream;

mod model;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An http error occurred")]
    Http (#[cause] reqwest::Error),
    #[fail(display = "An error occured while parsing a url")]
    Url(#[cause] url::ParseError),
    #[fail(display = "An error occured during a websocket operation {:?}",_0)]
    Ws(#[cause] tungstenite::error::Error),
    #[fail(display = "An error occured during (de)serialization")]
    Json(#[cause] serde_json::Error)
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

pub struct Connection{
    stream: Box<Stream<Item=model::ReceivablePayload,Error=Error> + Send>,
    sink: Box<Sink<SinkItem=model::SendablePayload,SinkError=Error>+Send>,
    heartbeat_interval: u64,
}


const API_BASE: &str = " https://discordapp.com/api/v6";

const GATEWAY_VERSION: u8 = 6;

pub fn get_gateway() -> Result<Url,Error>{
    #[derive(Deserialize)]
    struct GatewayResponse{
        url: String
    }
    let res: GatewayResponse = reqwest::get(&(API_BASE.to_owned()+"/gateway"))?.json()?;
    Ok(Url::parse(format!("{}?v={}&encoding=json",res.url,GATEWAY_VERSION).as_str())?)
}

pub fn connect_to_gateway(token: String) -> Box<Future<Item=Connection,Error=Error> + Send>{
    Box::new(futures::future::result(get_gateway()).and_then(|url|{
        tokio_tungstenite::connect_async(url).map_err(Error::from)
    }).map(|(stream,_res)|{
        let (sink,stream) = stream.split();
        let sink = Box::new(sink.sink_map_err(Error::from).with(|payload: model::SendablePayload|{
            let payload = model::Payload::try_from_sendable(payload)?;
            let payload = serde_json::to_string(&payload)?;
            println!("sending payload: {:?}",payload);
            Ok(tungstenite::Message::Text(payload))
        }));
        let stream = Box::new(stream.map_err(Error::from).and_then(|message|{
            let payload: model::Payload = serde_json::from_str(&message.into_text()?)?;
            Ok(payload.received_event_data()?)
        }));
        (sink,stream)
    }).and_then(|(sink,stream)|{
        stream.into_future().map(move|(payload,stream)| (sink,stream,payload.unwrap())).map_err(|(e,_strm)| Error::from(e))
    }).and_then(|(sink,stream,payload)|{
        println!("packet, should be hello: {:#?}",payload);
        let hello = payload.expect_hello();
        println!("{:#?}",hello);
        let identify = model::Identify::new(token);
        println!("sending identify payload: {:#?}",identify);
        Ok((sink,stream,hello,identify))
    }).and_then(|(sink,stream,hello,identify)|{
        sink.send(identify.into()).map(move |sink| (sink,stream,hello.heartbeat_interval)).map_err(Error::from)
    }).and_then(|(sink,stream,heartbeat_interval)|{
        stream.into_future().map(move |(message,stream)| (message,sink,stream,heartbeat_interval)).map_err(|(e,_strm)| Error::from(e))
    }).and_then(|(message,sink,stream,heartbeat_interval)|{
        stream.into_future().map(move|(payload,stream)| (sink,stream,payload.unwrap(),heartbeat_interval)).map_err(|(e,_strm)| Error::from(e))
    }).and_then(|(sink,stream,payload,heartbeat_interval)|{
        println!("packet, should be event ready: {:?}",payload);
        let ready = payload.expect_event().expect_ready();
        println!("{:?}",ready);
        Ok((sink,stream,heartbeat_interval))
    }).and_then(|(sink,stream,heartbeat_interval)|{
        Ok(Connection{sink,stream,heartbeat_interval})
    })) as Box<_>
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_gets_the_gateway() {
        assert!(get_gateway().is_ok());
    }
}
