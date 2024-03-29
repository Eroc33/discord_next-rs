use crate::model::{self, *};
use crate::Error;
use crate::API_BASE;
use itertools::Itertools;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use url::Url;

use tracing::*;

mod ratelimiter;
use ratelimiter::*;

#[derive(Clone)]
pub struct Client {
    http_client: reqwest::Client,
    bot_token: String,
    rate_limiter: RateLimiter,
    max_retries: u16,
}

#[derive(Default, Serialize)]
pub struct EditMessage {
    ///the message contents (up to 2000 characters)
    /// #[serde(default,skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    ///embedded rich content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embed: Option<model::Embed>,
    //TODO: flags
}

impl EditMessage {
    pub fn text<S: Into<String>>(s: S) -> Self {
        Self {
            content: Some(s.into()),
            ..Default::default()
        }
    }

    pub async fn update(
        self,
        channel_id: ChannelId,
        message_id: MessageId,
        client: &Client,
    ) -> Result<Message, Error> {
        Ok(client.update_message(channel_id, message_id, self).await?)
    }

    pub fn with_embed<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Embed),
    {
        self.embed = self.embed.take().or_else(|| Some(Default::default()));
        f(self
            .embed
            .as_mut()
            .expect("We just put a Some into it if it was empty"));
        self
    }

    pub fn enforce_embed_limits(&self) -> Result<(), EmbedTooBigError> {
        if let Some(embed) = self.embed.as_ref() {
            embed.enforce_embed_limits()?;
        }
        Ok(())
    }
}

#[derive(Default, Serialize)]
pub struct NewMessage {
    ///the message contents (up to 2000 characters)
    pub content: String,
    ///a nonce that can be used for optimistic message sending
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    ///true if this is a TTS message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    // ///the contents of the file being sent. Used in multipart requests (not currently supported)
    // pub file: Vec<u8>
    ///embedded rich content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embed: Option<model::Embed>, // ///JSON encoded body of any additional request fields. Used for multipart requests (not currently supported)
                                     // pub payload_json: String
}

#[derive(Debug, Clone, Copy)]
pub enum GetMessages {
    Around(MessageId),
    Before(MessageId),
    After(MessageId),
    MostRecent,
}

impl GetMessages {
    fn to_query(&self, query: &mut HashMap<&'static str, String>) {
        match self {
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

impl NewMessage {
    pub fn text<S: Into<String>>(s: S) -> Self {
        Self {
            content: s.into(),
            ..Default::default()
        }
    }

    pub async fn send(self, channel_id: ChannelId, client: &Client) -> Result<Message, Error> {
        Ok(client.send_message(channel_id, self).await?)
    }

    pub fn with_embed<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Embed),
    {
        self.embed = self.embed.take().or_else(|| Some(Default::default()));
        f(self
            .embed
            .as_mut()
            .expect("We just put a Some into it if it was empty"));
        self
    }

    pub fn enforce_embed_limits(&self) -> Result<(), EmbedTooBigError> {
        if let Some(embed) = self.embed.as_ref() {
            embed.enforce_embed_limits()?;
        }
        Ok(())
    }
}

impl Client {
    pub fn new<S: Into<String>>(bot_token: S) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            bot_token: bot_token.into(),
            max_retries: 5,
            rate_limiter: Default::default(),
        }
    }

    pub async fn update_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        edit_message: EditMessage,
    ) -> Result<Message, Error> {
        edit_message.enforce_embed_limits()?;
        let url = format!(
            "/channels/{channel_id}/messages/{message_id}",
            channel_id = (channel_id.0).0,
            message_id = (message_id.0).0
        );
        self.patch_return_json(None, url, edit_message).await
    }

    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        new_message: NewMessage,
    ) -> Result<Message, Error> {
        new_message.enforce_embed_limits()?;
        let url = format!(
            "/channels/{channel_id}/messages",
            channel_id = (channel_id.0).0
        );
        self.post_return_json(None, url, new_message).await
    }

    pub async fn get_guilds(&self) -> Result<Vec<PartialGuild>, Error> {
        self.get_json(None, "/users/@me/guilds").await
    }

    pub async fn get_guild_channels(&self, guild_id: GuildId) -> Result<Vec<Channel>, Error> {
        let url = format!("/guilds/{guild_id}/channels", guild_id = (guild_id.0).0);
        self.get_json(None, url).await
    }

    pub async fn create_private_channel(&self, recipient_id: UserId) -> Result<Channel, Error> {
        let url = "/users/@me/channels";
        self.post_return_json(None, url, json!({ "recipient_id": recipient_id }))
            .await
    }

    pub async fn get_messages(
        &self,
        channel_id: ChannelId,
        position: GetMessages,
        limit: Option<u32>,
    ) -> Result<Vec<Message>, Error> {
        let mut query = HashMap::new();
        position.to_query(&mut query);
        if let Some(limit) = limit {
            query.insert("limit", limit.to_string());
        }
        let query = query.iter().map(|(k, v)| [*k, &v[..]].join("=")).join("&");
        let query_string = if !query.is_empty() {
            format!("?{}", query)
        } else {
            String::new()
        };
        let limit_url = format!(
            "/channels/{channel_id}/messages",
            channel_id = (channel_id.0).0
        );
        let url = format!(
            "{limit_url}{query_string}",
            limit_url = limit_url,
            query_string = query_string
        );
        self.get_json(limit_url, url).await
    }

    pub async fn delete_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        //delete is not under the same rate limiter as other verbs, so we prepend delete to get a different limiter
        let limit_url = format!(
            "DELETE /channels/{channel_id}/messages/{{}}",
            channel_id = (channel_id.0).0
        );
        let url = format!(
            "/channels/{channel_id}/messages/{message_id}",
            channel_id = (channel_id.0).0,
            message_id = (message_id.0).0
        );
        self.delete(limit_url, url).await
    }

    pub async fn delete_messages<'a>(
        &'a self,
        channel_id: ChannelId,
        message_ids: &'a [MessageId],
    ) -> Result<(), Error> {
        //delete is not under the same rate limiter as other verbs, so we prepend delete to get a different limiter
        let limit_url = format!(
            "DELETE /channels/{channel_id}/messages/bulk_delete",
            channel_id = (channel_id.0).0
        );
        let url = format!(
            "/channels/{channel_id}/messages/bulk_delete",
            channel_id = (channel_id.0).0
        );
        self.post_json(limit_url, url, json!({ "messages": message_ids }))
            .await
    }

    async fn delete<S1, S2>(&self, limit_url: S1, url: S2) -> Result<(), Error>
    where
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        self.execute_request(
            reqwest::Method::DELETE,
            limit_url,
            url,
            reqwest::Body::from(&[] as &[u8]),
        )
        .await?;
        Ok(())
    }

    async fn get_json<T, S1, S2>(&self, limit_url: S1, url: S2) -> Result<T, Error>
    where
        T: DeserializeOwned + Unpin + 'static,
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        let res = self
            .execute_request(
                reqwest::Method::GET,
                limit_url,
                url,
                reqwest::Body::from(&[] as &[u8]),
            )
            .await?;
        Ok(res.json().await?)
    }

    async fn post_json<T, S1, S2>(&self, limit_url: S1, url: S2, data: T) -> Result<(), Error>
    where
        T: Serialize + 'static,
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        self.execute_request(
            reqwest::Method::POST,
            limit_url,
            url,
            reqwest::Body::from(serde_json::to_string(&data)?),
        )
        .await?;
        Ok(())
    }

    async fn patch_json<T, S1, S2>(&self, limit_url: S1, url: S2, data: T) -> Result<(), Error>
    where
        T: Serialize + 'static,
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        self.execute_request(
            reqwest::Method::PATCH,
            limit_url,
            url,
            reqwest::Body::from(serde_json::to_string(&data)?),
        )
        .await?;
        Ok(())
    }

    async fn post_return_json<R, T, S1, S2>(
        &self,
        limit_url: S1,
        url: S2,
        data: T,
    ) -> Result<R, Error>
    where
        T: Serialize + 'static,
        R: DeserializeOwned + Unpin + 'static,
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        let res = self
            .execute_request(
                reqwest::Method::POST,
                limit_url,
                url,
                reqwest::Body::from(serde_json::to_string(&data)?),
            )
            .await?;
        Ok(res.json().await?)
    }

    async fn patch_return_json<R, T, S1, S2>(
        &self,
        limit_url: S1,
        url: S2,
        data: T,
    ) -> Result<R, Error>
    where
        T: Serialize + 'static,
        R: DeserializeOwned + Unpin + 'static,
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        let res = self
            .execute_request(
                reqwest::Method::PATCH,
                limit_url,
                url,
                reqwest::Body::from(serde_json::to_string(&data)?),
            )
            .await?;
        Ok(res.json().await?)
    }

    async fn execute_request<S1, S2>(
        &self,
        method: reqwest::Method,
        limit_url: S1,
        url: S2,
        body: reqwest::Body,
    ) -> Result<reqwest::Response, Error>
    where
        S1: Into<Option<String>> + 'static,
        S2: AsRef<str> + 'static,
    {
        let mut retries = 0;
        let url = url.as_ref();
        let limit_url = limit_url.into().unwrap_or_else(|| url.to_owned());

        let absolute_url = format!("{base_url}{url}", base_url = API_BASE, url = url);

        let req_builder = self.http_client.request(method.clone(), &absolute_url);

        let mut req_builder = self.set_headers(req_builder);
        if body.as_bytes().map(|b| b.len()).unwrap_or(0) > 0 {
            req_builder = req_builder.header("Content-Type", "application/json");
        }
        let req_builder = req_builder.body(body);

        let req = req_builder.build()?;

        trace!("request: {:?}", req);

        'retry_loop: loop {
            self.rate_limiter.enforce_limit(&limit_url).await?;

            let send_req = req
                .try_clone()
                .expect("rest client can only use non-stream bodies due to retry requirements");

            let res = self.http_client.execute(send_req).await?;

            self.rate_limiter
                .update_limits(limit_url.clone(), res.headers());

            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if retries >= self.max_retries {
                    return Err(Error::TooManyRetries(self.max_retries, url.to_owned()));
                }
                retries += 1;
                continue 'retry_loop;
            }

            if !res.status().is_success() {
                let status = res.status();
                error!("Request failed with result: {:?}", res.text().await?);
                return Err(Error::UnsuccessfulHttp(status));
            }

            return Ok(res);
        }
    }

    fn set_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header(
                "Authorization",
                format!("Bot {bot_token}", bot_token = self.bot_token),
            )
            .header(
                "User-Agent",
                "discord-next-rs (github.com/Eroc33, 0.0.1-prototype)",
            )
    }

    pub async fn get_gateway(&self, version: u8) -> Result<Url, Error> {
        #[derive(Deserialize)]
        struct GatewayResponse {
            url: String,
        }
        let res: GatewayResponse = self.get_json(None, "/gateway").await?;
        Ok(Url::parse(
            format!("{}?v={}&encoding=json", res.url, version).as_str(),
        )?)
    }

    pub async fn get_application_commands(
        &self,
        application_id: ApplicationId,
    ) -> Result<Vec<ApplicationCommand>, Error> {
        let url = format!(
            "/applications/{application_id}/commands",
            application_id = (application_id.0).0
        );
        self.get_json(None, url).await
    }

    pub async fn create_application_command(
        &self,
        application_id: ApplicationId,
        command: NewApplicationCommand,
    ) -> Result<ApplicationCommand, Error> {
        let url = format!(
            "/applications/{application_id}/commands",
            application_id = (application_id.0).0
        );
        self.post_return_json(None, url, command).await
    }

    pub async fn delete_application_command(
        &self,
        application_id: ApplicationId,
        command_id: ApplicationCommandId,
    ) -> Result<(), Error> {
        let url = format!(
            "/applications/{application_id}/commands/{command_id}",
            application_id = (application_id.0).0,
            command_id = (command_id.0).0,
        );
        self.delete(None, url).await
    }

    pub async fn create_interaction_response(
        &self,
        interaction: &Interaction,
        response: InteractionResponse,
    ) -> Result<(), Error> {
        let url = format!(
            "/interactions/{interaction_id}/{interaction_token}/callback",
            interaction_id = (interaction.id.0).0,
            interaction_token = interaction.token,
        );
        self.post_json(None, url, response).await
    }

    pub async fn edit_original_interaction_response(
        &self,
        interaction: &Interaction,
        response: InteractionApplicationCommandCallbackData,
    ) -> Result<(), Error> {
        let url = format!(
            "/webhooks/{application_id}/{interaction_token}/messages/@original",
            application_id = (interaction.application_id.0).0,
            interaction_token = interaction.token,
        );
        self.patch_json(None, url, response).await
    }
}
