use serde::{Deserialize, Serialize};
use serde_json;

pub mod publisher_types;
pub mod types;

pub fn to_json_string<T>(val: &T) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_string(val)
}

pub fn from_json_str<T>(s: &str) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(s)
}
