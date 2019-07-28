use serde_json;
use serde::{Deserialize,Serialize};
use failure::Fail;

#[derive(Debug,Fail)]
pub enum FromPayloadError{
    #[fail(display = "Bad json for payload {:?}",_0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "Unknown opcode: {}",_0)]
    UnknownOpcode(u64),
}

impl From<serde_json::Error> for FromPayloadError{
    fn from(e: serde_json::Error) -> Self
    {
        FromPayloadError::Json(e)
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Payload{
    //opcode for the payload
    pub op: u64,
    //event data
    pub d: serde_json::Value,
    //sequence number, used for resuming sessions and heartbeats (Only for Opcode 0)
    pub s: Option<u64>,
    //the event name for this payload (Only for Opcode 0)	
    pub t: Option<String>,
}
