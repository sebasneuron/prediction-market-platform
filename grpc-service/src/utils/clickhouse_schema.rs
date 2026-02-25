use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Deserialize;
use sqlx::types::Uuid;

use crate::generated::markets::{MarketPrice, VolumeInfo};

#[derive(Row, Deserialize, Debug)]
pub struct GetMarketPrices {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    pub yes_price: f64,
    pub no_price: f64,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub ts: DateTime<Utc>,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub created_at: DateTime<Utc>,
}

pub type OrderBook = Vec<(f64, f64, u32)>;

#[derive(Row, Deserialize, Debug)]
pub struct GetOrderBook {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub ts: DateTime<Utc>,

    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub created_at: DateTime<Utc>,

    pub yes_bids: OrderBook,
    pub yes_asks: OrderBook,

    pub no_bids: OrderBook,
    pub no_asks: OrderBook,
}

#[derive(Row, Deserialize, Debug, Default)]
pub struct VolumeData {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,

    pub yes_buy_qty: f64,
    pub yes_buy_usd: f64,

    pub yes_sell_qty: f64,
    pub yes_sell_usd: f64,

    pub no_buy_qty: f64,
    pub no_buy_usd: f64,

    pub no_sell_qty: f64,
    pub no_sell_usd: f64,
}

#[derive(Row, Deserialize, Debug)]
pub struct MarketPriceResponse {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,

    pub latest_yes_price: f64,
    pub latest_no_price: f64,
}

impl From<MarketPriceResponse> for MarketPrice {
    fn from(data: MarketPriceResponse) -> Self {
        MarketPrice {
            market_id: data.market_id.to_string(),
            latest_yes_price: data.latest_yes_price,
            latest_no_price: data.latest_no_price,
        }
    }
}

impl Default for MarketPriceResponse {
    fn default() -> Self {
        MarketPriceResponse {
            market_id: Uuid::default(),
            latest_yes_price: 0.5,
            latest_no_price: 0.5,
        }
    }
}

impl From<VolumeData> for VolumeInfo {
    fn from(data: VolumeData) -> Self {
        VolumeInfo {
            market_id: data.market_id.to_string(),
            yes_buy_qty: data.yes_buy_qty,
            yes_buy_usd: data.yes_buy_usd,
            yes_sell_qty: data.yes_sell_qty,
            yes_sell_usd: data.yes_sell_usd,
            no_buy_qty: data.no_buy_qty,
            no_buy_usd: data.no_buy_usd,
            no_sell_qty: data.no_sell_qty,
            no_sell_usd: data.no_sell_usd,
        }
    }
}
