use serde::{Deserialize, Deserializer};

pub fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Number(u32),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<u32>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrInt::Number(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}
