use crate::Error;
use crate::model::{self,*};
use crate::{API_BASE};
use serde::{de::DeserializeOwned,Serialize};
use itertools::Itertools;
use url::Url;
use std::collections::HashMap;
use futures::stream::TryStreamExt;

use tracing::*;

mod ratelimiter;
use ratelimiter::*;

use hyper;
use hyper_tls::HttpsConnector;

#[derive(Clone)]
pub struct Client{
    http_client: hyper::Client<HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    bot_token: String,
    rate_limiter: RateLimiter,
    max_retries: u16
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
        client.send_message(channel_id,self).await?;
        Ok(())
    }

    pub fn with_embed<F>(mut self, f: F) -> Self
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
        //TODO: propagate error instead of using expect
        let https_connector = hyper_tls::HttpsConnector::new();
        Self{
            http_client: hyper::client::Client::builder().build(https_connector),
            bot_token: bot_token.into(),
            max_retries: 5,
            rate_limiter: Default::default(),
        }
    }

    pub async fn send_message(&self,channel_id: ChannelId, new_message: NewMessage) -> Result<(),Error>{
        new_message.enforce_embed_limits()?;
        let url = format!("/channels/{channel_id}/messages",channel_id=(channel_id.0).0);
        self.post_json(None,url,new_message).await
    }

    pub async fn get_guilds(&self) -> Result<Vec<PartialGuild>,Error>{
        self.get_json(None,"/users/@me/guilds").await
    }

    pub async fn get_guild_channels(&self, guild_id: GuildId) -> Result<Vec<Channel>,Error>{
        let url = format!("/guilds/{guild_id}/channels",guild_id=(guild_id.0).0);
        self.get_json(None,url).await
    }

    pub async fn create_private_channel(&self, recipient_id: UserId) -> Result<Channel,Error>{
        let url = "/users/@me/channels";
        self.post_return_json(None,url,json!({"recipient_id": recipient_id})).await
    }

    pub async fn get_messages(&self, channel_id: ChannelId, position: GetMessages, limit: Option<u32>) -> Result<Vec<Message>,Error>
    {
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
        let limit_url = format!("/channels/{channel_id}/messages",channel_id=(channel_id.0).0);
        let url = format!("{limit_url}{query_string}",limit_url=limit_url,query_string=query_string);
        self.get_json(limit_url,url).await
    }

    pub async fn delete_message(&self, channel_id: ChannelId, message_id: MessageId) -> Result<(),Error>{
        //delete is not under the same rate limiter as other verbs, so we prepend delete to get a different limiter
        let limit_url = format!("DELETE /channels/{channel_id}/messages/{{}}",channel_id=(channel_id.0).0);
        let url = format!("/channels/{channel_id}/messages/{message_id}", channel_id=(channel_id.0).0,message_id=(message_id.0).0);
        self.delete(limit_url,url).await
    }

    pub async fn delete_messages<'a>(&'a self, channel_id: ChannelId, message_ids: &'a [MessageId]) -> Result<(),Error>
    {
        //delete is not under the same rate limiter as other verbs, so we prepend delete to get a different limiter
        let limit_url = format!("DELETE /channels/{channel_id}/messages/bulk-delete",channel_id=(channel_id.0).0);
        let url = format!("/channels/{channel_id}/messages/bulk-delete",channel_id=(channel_id.0).0);
        self.post_json(limit_url,url,json!({"messages":message_ids})).await
    }

    async fn delete<S1,S2>(&self, limit_url: S1, url: S2) -> Result<(),Error>
        where S1: Into<Option<String>> + 'static,
              S2: AsRef<str> + 'static,
    {
        self.execute_request(hyper::Method::DELETE, limit_url, url, hyper::Body::empty()).await?;
        Ok(())
    }
    
    async fn get_json<T, S1, S2>(&self, limit_url: S1, url: S2) -> Result<T,Error>
        where T: DeserializeOwned + Unpin + 'static,
              S1: Into<Option<String>> + 'static,
              S2: AsRef<str> + 'static,
    {
        let res = self.execute_request(hyper::Method::GET, limit_url, url, hyper::Body::empty()).await?;
        let bytes = res.into_body().map_ok(|body| body.to_vec()).try_concat().await?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    async fn post_json<T, S1, S2>(&self, limit_url: S1, url: S2, data: T) -> Result<(),Error>
        where T: Serialize + 'static,
              S1: Into<Option<String>> + 'static,
              S2: AsRef<str> + 'static,
    {
        self.execute_request(hyper::Method::POST, limit_url, url, hyper::Body::from(serde_json::to_string(&data)?)).await?;
        Ok(())
    }

    async fn post_return_json<R, T, S1, S2>(&self, limit_url: S1, url: S2, data: T) -> Result<R,Error>
        where T: Serialize + 'static,
              R: DeserializeOwned + Unpin + 'static,
              S1: Into<Option<String>> + 'static,
              S2: AsRef<str> + 'static,
    {
        let res = self.execute_request(hyper::Method::POST, limit_url, url, hyper::Body::from(serde_json::to_string(&data)?)).await?;
        let bytes = res.into_body().map_ok(|body| body.to_vec()).try_concat().await?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    async fn execute_request<S1, S2>(&self, method: hyper::Method, limit_url: S1, url: S2, body: hyper::Body) -> Result<hyper::Response<hyper::Body>,Error>
        where S1: Into<Option<String>> + 'static,
              S2: AsRef<str> + 'static,
    {
        let mut retries = 0;
        let url = url.as_ref();
        let limit_url = limit_url.into().unwrap_or_else(|| url.to_owned());

        let body_bytes = body.map_ok(|body| body.to_vec()).try_concat().await?;

        'retry_loop: loop{
            self.rate_limiter.enforce_limit(&limit_url).await?;

            let absolute_url = format!("{base_url}{url}",base_url=API_BASE,url=url);

            let req_builder = hyper::Request::builder()
                .method(method.clone())
                .uri(&absolute_url);
            let mut req_builder = self.set_headers(req_builder);
            if body_bytes.len() > 0{
                req_builder = req_builder.header("Content-Type", "application/json");
            }
            let req = req_builder.body(hyper::Body::from(body_bytes.clone()))?;

            trace!("request: {:?}",req);

            let res = self.http_client.request(req).await?;

            self.rate_limiter.update_limits(limit_url.clone(),res.headers());

            if res.status() == hyper::StatusCode::TOO_MANY_REQUESTS{
                if retries >= self.max_retries{
                    return Err(Error::TooManyRetries(self.max_retries,url.to_owned()));
                }
                retries += 1;
                continue 'retry_loop;
            }

            if !res.status().is_success(){
                return Err(Error::UnsuccessfulHttp(res.status()))
            }

            return Ok(res)
        }
    }

    fn set_headers(&self, builder: http::request::Builder) -> http::request::Builder{
        builder
            .header("Authorization", format!("Bot {bot_token}",bot_token=self.bot_token))
            .header("User-Agent", "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)")
    }

    pub async fn get_gateway(&self, version: u8) -> Result<Url,Error>{
        #[derive(Deserialize)]
        struct GatewayResponse{
            url: String
        }
        let res: GatewayResponse = self.get_json(None,"/gateway").await?;
        Ok(Url::parse(format!("{}?v={}&encoding=json",res.url,version).as_str())?)
    }
}