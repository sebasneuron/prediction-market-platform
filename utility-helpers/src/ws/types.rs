use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    PriceUpdate(Uuid),
    PricePoster,
    OrderBookUpdate(Uuid),
    OrderBookPoster,
}

impl ChannelType {
    pub fn from_str(s: &str) -> Option<ChannelType> {
        if s.starts_with("price_update:") {
            let uuid_str = s.strip_prefix("price_update:");
            let uuid = Self::get_uuid_from_str(uuid_str);
            return match uuid {
                Some(uuid) => Some(ChannelType::PriceUpdate(uuid)),
                _ => None,
            };
        } else if s.starts_with("order_book_update:") {
            let uuid_str = s.strip_prefix("order_book_update:");
            let uuid = Self::get_uuid_from_str(uuid_str);
            return match uuid {
                Some(uuid) => Some(ChannelType::OrderBookUpdate(uuid)),
                _ => None,
            };
        } else if s.starts_with("price_poster") {
            return Some(ChannelType::PricePoster);
        } else if s.starts_with("order_book_poster") {
            return Some(ChannelType::OrderBookPoster);
        }
        None
    }

    pub fn to_str(&self) -> String {
        match self {
            ChannelType::PriceUpdate(uuid) => format!("price_update:{uuid}"),
            ChannelType::OrderBookUpdate(uuid) => format!("order_book_update:{uuid}"),
            ChannelType::PricePoster => "price_poster".to_string(),
            ChannelType::OrderBookPoster => "order_book_poster".to_string(),
        }
    }

    fn get_uuid_from_str(st: Option<&str>) -> Option<Uuid> {
        if let Some(uuid_str) = st {
            let uuid = Uuid::from_str(uuid_str);
            return match uuid {
                Ok(uuid) => Some(uuid),
                _ => None,
            };
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub id: Option<String>, //TODO we can verify the client id with this id (TODO for now)
    pub payload: MessagePayload,
}
