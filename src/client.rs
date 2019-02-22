use futures::prelude::*;
use crate::Error;
use crate::model::{self,*};
use crate::{API_BASE,GATEWAY_VERSION};
use reqwest;
use serde::{de::DeserializeOwned,Serialize};
use std::collections::HashMap;
use itertools::Itertools;
use url::Url;

#[derive(Clone)]
pub struct Client{
    http_client: reqwest::r#async::Client,
    bot_token: String,
}

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
    ///embedded rich content
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub embed: Option<model::Embed>
    // ///JSON encoded body of any additional request fields. Used for multipart requests (not currently supported)
    // pub payload_json: String
}

#[derive(Debug,Clone,Copy)]
pub enum GetMessages{
    Around(MessageId),
    Before(MessageId),
    After(MessageId),
    MostRecent
}

impl GetMessages{
    fn to_query(&self, query: &mut HashMap<&'static str,String>)
    {
        match self{
            GetMessages::Around(msg_id) => {
                query.insert("around", (msg_id.0).0.to_string());
            }
            GetMessages::Before(msg_id) => {
                query.insert("before", (msg_id.0).0.to_string());
            }
            GetMessages::After(msg_id) => {
                query.insert("after", (msg_id.0).0.to_string());
            }
            GetMessages::MostRecent => {
                //this is represented by the absence of the other fields
            }
        }
    }
}

impl NewMessage{
    pub fn text<S: Into<String>>(s:S) -> Self{
        Self{
            content: s.into(),
            ..Default::default()
        }
    }

    pub async fn send(self, channel_id: ChannelId, client: &Client) -> Result<(),Error>{
        await!(client.send_message(channel_id,self));
        Ok(())
    }

    pub fn with_embed<F>(mut self, mut f: F) -> Self
        where F: FnOnce(&mut Embed)
    {
        self.embed = self.embed.take().or_else(|| Some(Default::default()));
        f(self.embed.as_mut().expect("We just put a Some into it if it was empty"));
        self
    }

    pub fn enforce_embed_limits(&self) -> Result<(),EmbedTooBigError>{
        if let Some(embed) = self.embed.as_ref(){
            embed.enforce_embed_limits()?;
        }
        Ok(())
    }

}

impl Client{
    pub fn new<S: Into<String>>(bot_token: S) -> Self{
        Self{
            http_client: reqwest::r#async::Client::new(),
            bot_token: bot_token.into(),
        }
    }

    pub async fn send_message(&self,channel_id: ChannelId, new_message: NewMessage) -> Result<(),Error>{
        new_message.enforce_embed_limits()?;
        await!(self.post_json(format!("/channels/{channel_id}/messages",channel_id=(channel_id.0).0),new_message))
    }

    pub async fn get_guilds(&self) -> Result<Vec<Guild>,Error>{
        await!(self.get_json("/users/@me/guilds"))
    }

    pub async fn get_guild_channels(&self, guild_id: GuildId) -> Result<Vec<Channel>,Error>{
        await!(self.get_json(format!("/guilds/{guild_id}/channels",guild_id=(guild_id.0).0)))
    }

    pub async fn create_private_channel(&self, recipient_id: UserId) -> Result<Channel,Error>{
        await!(self.post_return_json("/users/@me/channels",json!({"recipient_id": recipient_id})))
    }

    pub async fn get_messages(&self, channel_id: ChannelId, position: GetMessages, limit: Option<u32>) -> Result<Vec<Message>,Error>{
        let mut query = HashMap::new();
        position.to_query(&mut query);
        if let Some(limit) = limit{
            query.insert("limit", limit.to_string());
        }
        let query = query.iter().map(|(k,v)| [*k,&v[..]].join("=")).join("&");
        let query_string = if !query.is_empty(){
            format!("?{}",query)
        }else{
            String::new()
        };
        await!(self.get_json(format!("/channels/{channel_id}/messages{query_string}",channel_id=(channel_id.0).0,query_string=query_string)))
    }

    pub async fn delete_message(&self, channel_id: ChannelId, message_id: MessageId) -> Result<(),Error>{
        await!(self.delete(format!("/channels/{channel_id}/messages/{message_id}", channel_id=(channel_id.0).0,message_id=(message_id.0).0)))
    }

    pub async fn delete_messages<'a>(&'a self, channel_id: ChannelId, message_ids: &'a [MessageId]) -> Result<(),Error>
    {
        await!(self.post_json(format!("/channels/{channel_id}/messages/bulk-delete",channel_id=(channel_id.0).0),json!({"messages":message_ids})))
    }

    async fn delete<S>(&self, url: S) -> Result<(),Error>
        where S: AsRef<str> + 'static,
    {
        let res = await!(self.http_client.delete(&format!("{base_url}{url}",base_url=API_BASE,url=url.as_ref()))
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
            .send())?;
        res.error_for_status()?;
        Ok(())
    }
    
    async fn get_json<T, S>(&self, url: S) -> Result<T,Error>
        where T: DeserializeOwned + Unpin + 'static,
              S: AsRef<str> + 'static,
    {
        let res = await!(self.http_client.get(&format!("{base_url}{url}",base_url=API_BASE,url=url.as_ref()))
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
            .send())?;
        Ok(await!(res.error_for_status()?.json())?)
    }

    async fn post_json<T, S>(&self, url: S, data: T) -> Result<(),Error>
        where T: Serialize + 'static,
              S: AsRef<str> + 'static,
    {
        let res = await!(self.http_client.post(&format!("{base_url}{url}",base_url=API_BASE,url=url.as_ref()))
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
            .json(&data)
            .send())?;
        res.error_for_status()?;
        Ok(())
    }

    async fn post_return_json<R, T, S>(&self, url: S, data: T) -> Result<R,Error>
        where T: Serialize + 'static,
              R: DeserializeOwned + Unpin + 'static,
              S: AsRef<str> + 'static,
    {
        let res = await!(self.http_client.post(&format!("{base_url}{url}",base_url=API_BASE,url=url.as_ref()))
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
            .json(&data)
            .send())?;
        Ok(await!(res.error_for_status()?.json())?)
    }

    pub async fn get_gateway(&self) -> Result<Url,Error>{
        #[derive(Deserialize)]
        struct GatewayResponse{
            url: String
        }
        let res: GatewayResponse = await!(self.get_json("/gateway"))?;
        Ok(Url::parse(format!("{}?v={}&encoding=json",res.url,GATEWAY_VERSION).as_str())?)
    }
}