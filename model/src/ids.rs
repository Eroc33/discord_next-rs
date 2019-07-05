use serde::{de,Deserialize,Serialize};

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct Snowflake(
    #[serde(deserialize_with="u64_from_string")]
    pub u64
);

fn u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use de::{Visitor,Unexpected};

    struct V;

    impl<'de> Visitor<'de> for V{
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
            write!(f,"A snowflake (u64 as a string)")
        }

        fn visit_str<E>(self,s: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            match u64::from_str_radix(s,10){
                Ok(o) => Ok(o),
                Err(_e) => Err(de::Error::invalid_value(Unexpected::Str(s),&self))
            }
        }
    }

    deserializer.deserialize_str(V)
}

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct ChannelId(pub Snowflake);

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct RoleId(pub Snowflake);

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct GuildId(pub Snowflake);

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct MessageId(pub Snowflake);

#[derive(Debug,Deserialize,Serialize,PartialEq,Eq,Clone,Copy)]
#[serde(transparent)]
pub struct UserId(pub Snowflake);