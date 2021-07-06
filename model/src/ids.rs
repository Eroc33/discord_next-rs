use serde::{Deserialize, Serialize};

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct Snowflake(
    #[serde(deserialize_with = "crate::custom_serialization::u64_from_string")] pub u64,
);

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct ChannelId(pub Snowflake);

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct RoleId(pub Snowflake);

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct GuildId(pub Snowflake);

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct MessageId(pub Snowflake);

#[derive(Hash, Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct UserId(pub Snowflake);
