// these `ORDER` name does not indicate the operations on orders, instead it indicates that the streams is used by order-service microservice, so don't confuse it with the order name and same for it's topics, all topics are prefixed with `order.`

pub mod types;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatsSubjects {
    OrderCreate,
    MarketBookUpdate(Uuid),
    OrderCancel,
    OrderUpdate,
    MarketOrderCreate,
    InitializeOrderBook,
    FinalizeMarket,
}

impl NatsSubjects {
    pub fn to_string(&self) -> String {
        match self {
            NatsSubjects::OrderCreate => "order.create".to_string(),
            NatsSubjects::MarketBookUpdate(market_id) => {
                format!("order.market.book.update.{}", market_id)
            }
            NatsSubjects::OrderCancel => "order.cancel".to_string(),
            NatsSubjects::OrderUpdate => "order.update".to_string(),
            NatsSubjects::MarketOrderCreate => "order.market_order_create".to_string(),
            NatsSubjects::InitializeOrderBook => "order.initialize_order_book".to_string(),
            NatsSubjects::FinalizeMarket => "order.finalize_market".to_string(),
        }
    }

    pub fn from_string(queue: &str) -> Option<Self> {
        if queue == "order.create" {
            Some(NatsSubjects::OrderCreate)
        } else if queue.starts_with("order.market.book.update.") {
            let market_id_str = queue.trim_start_matches("order.market.book.update.");
            let res_uuid = Uuid::parse_str(market_id_str);
            match res_uuid {
                Ok(market_id) => Some(NatsSubjects::MarketBookUpdate(market_id)),
                Err(_) => None,
            }
        } else if queue == "order.cancel" {
            Some(NatsSubjects::OrderCancel)
        } else if queue == "order.update" {
            Some(NatsSubjects::OrderUpdate)
        } else if queue == "order.market_order_create" {
            Some(NatsSubjects::MarketOrderCreate)
        } else if queue == "order.initialize_order_book" {
            Some(NatsSubjects::InitializeOrderBook)
        } else if queue == "order.finalize_market" {
            Some(NatsSubjects::FinalizeMarket)
        } else {
            None
        }
    }
}

impl std::fmt::Display for NatsSubjects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl std::str::FromStr for NatsSubjects {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NatsSubjects::from_string(s).ok_or_else(|| format!("Invalid Nats queue: {}", s))
    }
}
