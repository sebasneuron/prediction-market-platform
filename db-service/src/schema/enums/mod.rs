use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"market_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum MarketStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN = 1,
    #[serde(rename = "closed")]
    CLOSED = 2,
    #[serde(rename = "settled")]
    SETTLED = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"outcome\"")]
#[sqlx(rename_all = "lowercase")]
pub enum Outcome {
    #[serde(rename = "yes")]
    YES = 1,
    #[serde(rename = "no")]
    NO = 2,
    #[default]
    #[serde(rename = "unspecified")]
    UNSPECIFIED = 0,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"order_side\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderSide {
    #[default]
    #[serde(rename = "buy")]
    BUY = 1, // bids
    #[serde(rename = "sell")]
    SELL = 2, // asks
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN = 1,
    #[serde(rename = "filled")]
    FILLED = 2,
    #[serde(rename = "cancelled")]
    CANCELLED = 3,
    #[serde(rename = "expired")]
    EXPIRED = 4,
    #[serde(rename = "unspecified")]
    UNSPECIFIED = 5,
    #[sqlx(rename = "pending_update")]
    PendingUpdate = 6,
    #[sqlx(rename = "pending_cancel")]
    PendingCancel = 7,
    // NOT USED!!!! and DON'T USE IT
    #[sqlx(rename = "partial_fill")]
    PartialFill = 8,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionType {
    #[default]
    #[serde(rename = "deposit")]
    DEPOSIT = 1,
    #[serde(rename = "withdrawal")]
    WITHDRAWAL = 2,
    #[serde(rename = "trade")]
    TRADE = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionStatus {
    #[default]
    #[serde(rename = "pending")]
    PENDING = 1,
    #[serde(rename = "completed")]
    COMPLETED = 2,
    #[serde(rename = "failed")]
    FAILED = 3,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderType {
    #[default]
    #[serde(rename = "limit")]
    LIMIT = 1,
    #[serde(rename = "market")]
    MARKET = 2,
    #[serde(rename = "stop_loss")]
    StopLoss = 3,
    #[serde(rename = "take_profit")]
    TakeProfit = 4,
}
