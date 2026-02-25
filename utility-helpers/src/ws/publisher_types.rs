use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::ws::types::ChannelType;

#[derive(Debug)]
pub struct GenericWrapper<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub channel: ChannelType,
    pub data: T,
}

impl<T> GenericWrapper<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn wrap_with_channel(channel: ChannelType, data: T) -> Self {
        GenericWrapper { channel, data }
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        let channel_str = self.channel.to_str();

        // let channel_str = self.channel.to_str();
        let final_data = json!({
            "payload":{
                "channel": channel_str,
                "params": self.data,
            }
        })
        .to_string();

        Ok(final_data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PricePosterDataStruct {
    pub market_id: Uuid,
    pub yes_price: Decimal,
    pub no_price: Decimal,
}

// REUSABLE TYPES ACROSS SERVICES --- till here
// and send websocket message from order service
