use futures::prelude::*;
use crate::Error;
use crate::model;
use reqwest;

#[derive(Clone)]
pub struct Client{
    http_client: reqwest::r#async::Client,
    bot_token: String,
}

const API_BASE: &str = "https://discordapp.com/api/v6";

#[derive(Default,Serialize)]
pub struct NewMessage{
    ///the message contents (up to 2000 characters)
    pub content: String,
    ///a nonce that can be used for optimistic message sending
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,	
    ///true if this is a TTS message
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    // ///the contents of the file being sent. Used in multipart requests (not currently supported)
    // pub file: Vec<u8>
    // ///embedded rich content (not currently supported)
    // #[serde(default,skip_serializing_if = "Option::is_none")]
    // pub embed: Option<model::Embed>
    // ///JSON encoded body of any additional request fields. Used for multipart requests (not currently supported)
    // pub payload_json: String
}

impl Client{
    pub fn new(bot_token: String) -> Self{
        Self{
            http_client: reqwest::r#async::Client::new(),
            bot_token
        }
    }

    pub async fn create_message(&self,channel_id: model::ChannelId, new_message: NewMessage) -> Result<(),Error>{
        let res = await!(self.http_client.post(&format!("{base_url}/channels/{channel_id}/messages",base_url=API_BASE,channel_id=(channel_id.0).0))
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
            .json(&new_message)
            .send())?;
        res.error_for_status()?;
        Ok(())
    }
}