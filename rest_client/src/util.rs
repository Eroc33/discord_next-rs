use crate::*;
use crate::model::{ChannelId,Message};
use async_trait::async_trait;

#[async_trait]
pub trait MessageExt{
    async fn create_or_update_text<S: Into<String> + Send>(self, s:S, channel_id: ChannelId, client: &Client) -> Result<Message,Error>;
}

#[async_trait]
impl MessageExt for Option<Message>{
    async fn create_or_update_text<S: Into<String> + Send>(self, s:S, channel_id: ChannelId, client: &Client) -> Result<Message,Error>{
        Ok(if let Some(message) = self{
            EditMessage::text(s).update(channel_id, message.id, client).await?
        }else{
            NewMessage::text(s).send(channel_id, client).await?
        })
    }
}