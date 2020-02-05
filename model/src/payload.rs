use serde_json;
use serde::{Deserialize,Serialize};
use thiserror::Error;

#[derive(Debug,Error)]
pub enum FromPayloadError{
    #[error("Bad json for payload {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("Unknown opcode: {0}")]
    UnknownOpcode(u64),
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
