use serde::{de, Deserialize, Serialize};

pub fn u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use de::{Unexpected, Visitor};

    struct V;

    impl<'de> Visitor<'de> for V {
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "A snowflake (u64 as a string)")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match u64::from_str_radix(s, 10) {
                Ok(o) => Ok(o),
                Err(_e) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
            }
        }
    }

    deserializer.deserialize_str(V)
}
