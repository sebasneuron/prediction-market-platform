use db_service::{
    pagination::PageInfo as DbPageInfo,
    schema::{
        market::Market as DbMarket, user_holdings::UserIdWithShares, user_trades::MarketTrades,
    },
};
use utility_helpers::to_f64_verbose;

use crate::{
    generated::{
        common::PageInfo,
        markets::{
            GetMarketBookResponse, Market, MarketTrade as GeneratedMarketTrade, OrderBook,
            OrderLevel, UserWithTotalHoldings,
        },
    },
    utils::clickhouse_schema::GetOrderBook,
};

pub mod market_services;
pub mod price_services;

// all type conversations.....

pub fn from_db_market(value: &DbMarket, yes_price: f32, no_price: f32) -> Market {
    Market {
        created_at: value.created_at.to_string(),
        description: value.description.clone(),
        final_outcome: value.final_outcome as i32,
        id: value.id.to_string(),
        liquidity_b: to_f64_verbose(value.liquidity_b),
        logo: value.logo.clone(),
        name: value.name.clone(),
        status: value.status as i32,
        updated_at: value.updated_at.to_string(),
        market_expiry: value.market_expiry.to_string(),
        no_price,
        yes_price,
    }
}

impl From<DbPageInfo> for PageInfo {
    fn from(value: DbPageInfo) -> Self {
        PageInfo {
            page: value.page,
            page_size: value.page_size,
            total_items: value.total_items,
            total_pages: value.total_pages,
        }
    }
}
impl From<MarketTrades> for GeneratedMarketTrade {
    fn from(value: MarketTrades) -> Self {
        GeneratedMarketTrade {
            created_at: value.timestamp.to_string(),
            email: value.email,
            avatar: value.avatar,
            id: value.id.to_string(),
            name: value.name,
            outcome: value.outcome as i32,
            price: to_f64_verbose(value.price),
            quantity: to_f64_verbose(value.quantity),
            trade_type: value.trade_type as i32,
        }
    }
}

impl From<UserIdWithShares> for UserWithTotalHoldings {
    fn from(value: UserIdWithShares) -> Self {
        UserWithTotalHoldings {
            user_id: value.user_id.to_string(),
            total_shares: value
                .total_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            total_yes_shares: value
                .total_yes_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            total_no_shares: value
                .total_no_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            username: value.username.unwrap_or_default(),
            avatar: value.avatar.unwrap_or_default(),
        }
    }
}

pub fn to_resp_for_market_book(data: GetOrderBook) -> GetMarketBookResponse {
    GetMarketBookResponse {
        market_id: data.market_id.to_string(),
        yes_book: Some(OrderBook {
            bids: data
                .yes_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .yes_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
        no_book: Some(OrderBook {
            bids: data
                .no_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .no_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
    }
}
