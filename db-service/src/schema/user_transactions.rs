use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use super::enums::{UserTransactionStatus, UserTransactionType};

#[derive(Debug, sqlx::FromRow, Default, Serialize)]
pub struct UserTransactions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub transaction_type: UserTransactionType,
    pub transaction_status: UserTransactionStatus,
    pub tx_hash: String,
    pub confirmed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserTransactions {
    pub async fn create_user_transaction(
        pg_pool: &sqlx::PgPool,
        user_id: Uuid,
        amount: Decimal,
        transaction_type: UserTransactionType,
        transaction_status: UserTransactionStatus,
        tx_hash: String,
    ) -> Result<UserTransactions, sqlx::error::Error> {
        let transaction = sqlx::query_as!(
            UserTransactions,
            r#"
            INSERT INTO polymarket.user_transactions (user_id, amount, transaction_type, transaction_status, tx_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, amount, transaction_type as "transaction_type: UserTransactionType", transaction_status as "transaction_status: UserTransactionStatus", tx_hash, confirmed_at, created_at, updated_at
            "#,
            user_id,
            amount,
            transaction_type as UserTransactionType,
            transaction_status as UserTransactionStatus,
            tx_hash
        )
        .fetch_one(pg_pool)
        .await?;

        Ok(transaction)
    }
}
