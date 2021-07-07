use serde::{Deserialize, Serialize};

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct Snowflake(
    #[serde(deserialize_with = "crate::custom_serialization::u64_from_string")] pub u64,
);

macro_rules! define_typed_ids {
    ($($name:ident,)+) => {
        $(
            #[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
            #[serde(transparent)]
            pub struct $name(pub Snowflake);
        )+
    };
}

define_typed_ids! {
    ChannelId,
    RoleId,
    GuildId,
    MessageId,
    UserId,
    ApplicationId,
    StickerId,
    EmojiId,
    TeamId,
    ApplicationCommandId,
    StageInstanceId,
    WebhookId,
    InteractionId,
}
