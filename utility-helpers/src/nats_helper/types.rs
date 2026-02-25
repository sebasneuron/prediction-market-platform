/*
 * This file contains types which are going to serialize using message pack pack and send to nats
 */

use proto_defs::proto_types::order_book::{MarketBook, OrderBook, OrderLevel};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    to_f64,
    types::{OrderBookDataStruct, OrderLevel as OrderLevelStruct},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBookUpdateData {
    pub yes_book: OrderBookDataStruct,
    pub no_book: OrderBookDataStruct,
    pub market_id: Uuid,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderMessage {
    pub order_id: Uuid,
    pub new_quantity: Decimal,
    pub new_price: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderCreateMessage {
    pub order_id: Uuid,
    pub budget: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: serde::de::DeserializeOwned"
))]
pub struct InitializeOrderBookMessage<T> {
    pub liquidity_b: Decimal,
    pub orders: Vec<T>,
}

impl OrderBookUpdateData {
    pub fn get_prost_market_book(self, market_id: Uuid) -> MarketBook {
        let yes_book_bids = Self::get_order_level(&self.yes_book.bids);

        let yes_book_asks = Self::get_order_level(&self.yes_book.asks);
        let no_book_bids = Self::get_order_level(&self.no_book.bids);
        let no_book_asks = Self::get_order_level(&self.no_book.asks);

        let yes_book = OrderBook {
            bids: yes_book_bids,
            asks: yes_book_asks,
        };
        let no_book = OrderBook {
            bids: no_book_bids,
            asks: no_book_asks,
        };

        MarketBook {
            market_id: market_id.to_string(),
            yes_book: Some(yes_book),
            no_book: Some(no_book),
        }
    }

    fn get_order_level(order_level: &Vec<OrderLevelStruct>) -> Vec<OrderLevel> {
        order_level
            .iter()
            .map(|level| OrderLevel {
                price: to_f64(level.price).unwrap_or_default(),
                shares: to_f64(level.shares).unwrap_or_default(),
                users: level.users as u32,
            })
            .collect()
    }
}
