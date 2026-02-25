use base64::{Engine, engine::general_purpose::STANDARD as base64_engine};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use utility_helpers::{log_info, symmetric::encrypt, types::GoogleClaims};

use crate::schema::enums::OrderSide;

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct User {
    pub id: Uuid,

    // oAuth2 fields
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar: String,
    pub last_login: NaiveDateTime,

    // wallet fields
    pub public_key: String,
    pub private_key: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub balance: Decimal,
}

#[derive(Debug, Serialize, Default)]
pub struct UserBalance {
    pub balance: Decimal,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct UserProfileInsights {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub public_key: String,
    pub balance: Decimal,
    pub last_login: NaiveDateTime,
    pub created_at: NaiveDateTime,

    // Orders
    pub open_orders: Option<i64>,
    pub partial_orders: Option<i64>,
    pub total_orders: Option<i64>,
    pub avg_fill_ratio: Option<Decimal>,

    // Trades
    pub total_trades: Option<i64>,
    pub total_volume: Option<Decimal>,
    pub avg_trade_price: Option<Decimal>,
    pub max_trade_qty: Option<Decimal>,
    pub first_trade_at: Option<NaiveDateTime>,
    pub last_trade_at: Option<NaiveDateTime>,
    pub markets_traded: Option<i64>,

    // Transactions
    pub total_deposit: Option<Decimal>,
    pub total_withdraw: Option<Decimal>,
    pub last_deposit: Option<NaiveDateTime>,
    pub last_withdraw: Option<NaiveDateTime>,
}

impl User {
    pub async fn create_new_user(
        pool: &PgPool,
        claims: &GoogleClaims,
    ) -> Result<Self, sqlx::Error> {
        let new_key_pair = Keypair::new();

        let private_key = new_key_pair.to_base58_string();
        let public_key = new_key_pair.pubkey().to_string();

        let encrypted_private_key_bytes = encrypt(private_key.as_bytes())
            .map_err(|_| sqlx::Error::Decode("Failed to encrypt private key".into()))?;
        let encrypted_private_key = base64_engine.encode(encrypted_private_key_bytes);

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO "polymarket"."users" (
                google_id,
                email,
                name,
                avatar,
                public_key, 
                private_key
            ) VALUES (
                $1, $2, $3, $4, $5, $6
            ) RETURNING *                
            "#,
            claims.sub,
            claims.email,
            claims.name,
            claims.picture,
            public_key,
            encrypted_private_key
        )
        .fetch_one(pool)
        .await?;

        log_info!("User added {}", user.id);

        Ok(user)
    }

    pub async fn create_or_update_existing_user(
        pool: &PgPool,
        claims: &GoogleClaims,
    ) -> Result<(Self, bool), sqlx::Error> {
        let existing_user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "polymarket"."users" WHERE google_id = $1
            "#,
            claims.sub
        )
        .fetch_optional(pool)
        .await?;

        if let Some(user) = existing_user {
            let updated_user = sqlx::query_as!(
                User,
                r#"
                UPDATE "polymarket"."users" SET
                    email = $1,
                    name = $2,
                    avatar = $3,
                    last_login = CURRENT_TIMESTAMP
                WHERE id = $4
                RETURNING *
                "#,
                claims.email,
                claims.name,
                claims.picture,
                user.id
            )
            .fetch_one(pool)
            .await?;

            log_info!("User updated {}", updated_user.id);
            Ok((updated_user, false))
        } else {
            // Create a new user

            let new_user = Self::create_new_user(pool, claims).await?;
            Ok((new_user, true))
        }
    }

    pub async fn get_user_by_id<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "polymarket"."users" WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(user)
    }

    pub async fn get_user_balance(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let balance = sqlx::query_as!(
            UserBalance,
            r#"
            SELECT balance FROM polymarket.users WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(balance.balance)
    }

    pub async fn get_two_users_balance<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_1_id: Uuid,
        user_2_id: Uuid,
    ) -> Result<(Decimal, Decimal), sqlx::Error> {
        let balances = sqlx::query_as!(
            UserBalance,
            r#"
            SELECT balance from polymarket.users where id in (
                $1, $2
            );
            "#,
            user_1_id,
            user_2_id
        )
        .fetch_all(executor)
        .await?;

        if balances.len() != 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        let user_1_balance = balances[0].balance;
        let user_2_balance = balances[1].balance;
        Ok((user_1_balance, user_2_balance))
    }

    pub async fn update_two_users_balance<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_1_id: Uuid,
        user_2_id: Uuid,
        balance_to_update: Decimal,
        user_1_side: OrderSide,
    ) -> Result<(), sqlx::Error> {
        // user_1_side buy then current balance + user_1_new_balance else current balance - user_1_new_balance
        // user_2_side buy then current balance + user_2_new_balance else current balance - user_2_new_balance
        sqlx::query!(
            r#"
            UPDATE polymarket.users
            SET balance = CASE
                WHEN id = $1 THEN balance + ($2::numeric * (CASE WHEN $3 = 'sell'::polymarket.order_side THEN 1 ELSE -1 END))
                WHEN id = $4 THEN balance + ($2::numeric * (CASE WHEN $3 = 'buy'::polymarket.order_side THEN 1 ELSE -1 END))
            END
            WHERE id IN ($1, $4);
            "#,
            user_1_id,
            balance_to_update,
            user_1_side as _,
            user_2_id,
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn get_all_user_ids(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
        let user_ids = sqlx::query!(
            r#"
            SELECT id FROM "polymarket"."users"
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(user_ids.into_iter().map(|u| u.id).collect())
    }

    pub async fn get_or_create_admin(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let admin_email = "admin@admin.com";
        let admin_name = "Admin";
        let admin_avatar = "https://your-domain.com/images/logo.png";
        let admin_google_id = "admin_google_id";
        let admin_balance = Decimal::new(1_000_000, 2); // 10,000.00

        let admin = sqlx::query_as!(
            User,
            r#"
            INSERT INTO "polymarket"."users" (
                google_id,
                email,
                name,
                avatar,
                public_key, 
                private_key,
                balance
            ) VALUES (
                $1, $2, $3, $4, 'no_puk', 'no_prk', $5
            ) ON CONFLICT (google_id) DO UPDATE SET
                email = EXCLUDED.email,
                name = EXCLUDED.name,
                avatar = EXCLUDED.avatar,
                last_login = CURRENT_TIMESTAMP,
                balance = EXCLUDED.balance
            RETURNING *
            "#,
            admin_google_id,
            admin_email,
            admin_name,
            admin_avatar,
            admin_balance
        )
        .fetch_one(pool)
        .await?;

        Ok(admin)
    }

    pub async fn get_user_metadata(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<UserProfileInsights, sqlx::Error> {
        let user = sqlx::query_as!(
            UserProfileInsights,
            r#"
                WITH 
                holdings AS (
                    SELECT 
                        uh.market_id,
                        uh.outcome,
                        uh.shares
                    FROM polymarket.user_holdings uh
                    WHERE uh.user_id = $1
                ),

                orders AS (
                    SELECT 
                        COUNT(*) FILTER (WHERE status = 'open') AS open_orders,
                        COUNT(*) FILTER (WHERE status = 'partial_fill') AS partial_orders,
                        COUNT(*) AS total_orders,
                        AVG(filled_quantity / NULLIF(quantity, 0)) AS avg_fill_ratio
                    FROM polymarket.orders
                    WHERE user_id = $1
                ),

                trades AS (
                    SELECT 
                        COUNT(*) AS total_trades,
                        SUM(quantity) AS total_volume,
                        AVG(price) AS avg_trade_price,
                        MAX(quantity) AS max_trade_qty,
                        MIN(created_at) AS first_trade_at,
                        MAX(created_at) AS last_trade_at,
                        COUNT(DISTINCT market_id) AS markets_traded
                    FROM polymarket.user_trades
                    WHERE user_id = $1
                ),

                txns AS (
                    SELECT
                        SUM(amount) FILTER (WHERE transaction_type = 'deposit') AS total_deposit,
                        SUM(amount) FILTER (WHERE transaction_type = 'withdrawal') AS total_withdraw,
                        MAX(created_at) FILTER (WHERE transaction_type = 'deposit') AS last_deposit,
                        MAX(created_at) FILTER (WHERE transaction_type = 'withdrawal') AS last_withdraw
                    FROM polymarket.user_transactions
                    WHERE user_id = $1
                )

                SELECT
                    u.id,
                    u.name,
                    u.email,
                    u.avatar,
                    u.public_key,
                    u.balance,
                    u.last_login,
                    u.created_at,
                    
                    -- Orders
                    o.open_orders,
                    o.partial_orders,
                    o.total_orders,
                    o.avg_fill_ratio,

                    -- Trades
                    t.total_trades::bigint,
                    t.total_volume,
                    t.avg_trade_price,
                    t.max_trade_qty,
                    t.first_trade_at,
                    t.last_trade_at,
                    t.markets_traded::bigint,

                    -- Txns
                    x.total_deposit,
                    x.total_withdraw,
                    x.last_deposit,
                    x.last_withdraw

                FROM polymarket.users u
                LEFT JOIN orders o ON true
                LEFT JOIN trades t ON true
                LEFT JOIN txns x ON true
                WHERE u.id = $1;
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use utility_helpers::symmetric::decrypt;

    use super::*;

    async fn cleanup_test_user(pool: &PgPool, user_id: Uuid) {
        sqlx::query(r#"DELETE FROM "polymarket"."users" WHERE id = $1"#)
            .bind(user_id)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_new_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3, // 3 days,
            name: "Test User".to_string(),
            picture: "https://example.com/avatar.png".to_string(),
        };

        let user = User::create_new_user(&pool, &google_claims).await.unwrap();

        let decoded_private_key = base64_engine.decode(&user.private_key).unwrap();
        let decrypted_private_key = decrypt(&decoded_private_key).unwrap();
        let _decrypted_private_key_str = String::from_utf8(decrypted_private_key).unwrap();

        assert!(!user.private_key.is_empty());
        assert!(!user.public_key.is_empty());
        assert_eq!(user.name, "Test User");
        assert_eq!(user.avatar, "https://example.com/avatar.png");
        assert_eq!(user.balance, Decimal::ZERO);
        assert_eq!(user.created_at, user.updated_at);

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_or_update_existing_user_new_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User New".to_string(),
            picture: "https://example.com/avatar_new.png".to_string(),
        };

        let (user, is_new) = User::create_or_update_existing_user(&pool, &google_claims)
            .await
            .unwrap();

        // Verify it's a new user
        assert!(is_new);
        assert_eq!(user.name, "Test User New");
        assert_eq!(user.avatar, "https://example.com/avatar_new.png");
        assert_eq!(user.google_id, google_claims.sub);
        assert_eq!(user.email, google_claims.email);

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_or_update_existing_user_existing_user() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        // First create a user
        let google_claims_initial = GoogleClaims {
            sub: unique_sub.clone(),
            email: unique_email.clone(),
            exp: 60 * 60 * 24 * 3,
            name: "Test User Initial".to_string(),
            picture: "https://example.com/avatar_initial.png".to_string(),
        };

        let (initial_user, is_new_initial) =
            User::create_or_update_existing_user(&pool, &google_claims_initial)
                .await
                .unwrap();
        assert!(is_new_initial);

        // Now update the user
        let google_claims_updated = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User Updated".to_string(),
            picture: "https://example.com/avatar_updated.png".to_string(),
        };

        let (updated_user, is_new_updated) =
            User::create_or_update_existing_user(&pool, &google_claims_updated)
                .await
                .unwrap();

        // Verify it's an updated user
        assert!(!is_new_updated);
        assert_eq!(updated_user.id, initial_user.id); // Same ID
        assert_eq!(updated_user.name, "Test User Updated"); // Name updated
        assert_eq!(
            updated_user.avatar,
            "https://example.com/avatar_updated.png"
        ); // Avatar updated

        // Keys should remain the same
        assert_eq!(updated_user.public_key, initial_user.public_key);
        assert_eq!(updated_user.private_key, initial_user.private_key);

        // Clean up
        cleanup_test_user(&pool, initial_user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3,
            name: "Test User Get".to_string(),
            picture: "https://example.com/avatar_get.png".to_string(),
        };

        // Create user first
        let created_user = User::create_new_user(&pool, &google_claims).await.unwrap();

        // Get user by ID
        let fetched_user = User::get_user_by_id(&pool, created_user.id).await.unwrap();

        // Verify fetched user matches created user
        assert_eq!(fetched_user.id, created_user.id);
        assert_eq!(fetched_user.name, created_user.name);
        assert_eq!(fetched_user.email, created_user.email);
        assert_eq!(fetched_user.public_key, created_user.public_key);
        assert_eq!(fetched_user.private_key, created_user.private_key);

        // Clean up
        cleanup_test_user(&pool, created_user.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_user_by_id_nonexistent() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Generate a random UUID that shouldn't exist in the database
        let nonexistent_id = Uuid::new_v4();

        // Attempt to get user by non-existent ID
        let result = User::get_user_by_id(&pool, nonexistent_id).await;

        // Verify the error is RowNotFound
        assert!(result.is_err());
        match result {
            Err(sqlx::Error::RowNotFound) => (),
            _ => panic!("Expected RowNotFound error"),
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_two_users_balance() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create two users
        let user1_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 1".to_string(),
            picture: "https://example.com/avatar1.png".to_string(),
        };

        let user2_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 2".to_string(),
            picture: "https://example.com/avatar2.png".to_string(),
        };

        let user1 = User::create_new_user(&pool, &user1_claims).await.unwrap();
        let user2 = User::create_new_user(&pool, &user2_claims).await.unwrap();

        // Get balances
        let (user1_balance, user2_balance) = User::get_two_users_balance(&pool, user1.id, user2.id)
            .await
            .unwrap();

        // Verify initial balances are zero
        assert_eq!(user1_balance, Decimal::ZERO);
        assert_eq!(user2_balance, Decimal::ZERO);

        // Clean up
        cleanup_test_user(&pool, user1.id).await;
        cleanup_test_user(&pool, user2.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_two_users_balance_one_nonexistent() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Create one user
        let user1_claims = GoogleClaims {
            sub: format!("test_google_id_{}", Uuid::new_v4()),
            email: format!("test_{}@gmail.com", Uuid::new_v4()),
            exp: 60 * 60 * 24 * 3,
            name: "Test User 1".to_string(),
            picture: "https://example.com/avatar1.png".to_string(),
        };

        let user1 = User::create_new_user(&pool, &user1_claims).await.unwrap();
        let nonexistent_id = Uuid::new_v4();

        // Get balances with one nonexistent user
        let result = User::get_two_users_balance(&pool, user1.id, nonexistent_id).await;

        // Should fail since one user doesn't exist
        assert!(result.is_err());
        match result {
            Err(sqlx::Error::RowNotFound) => (),
            _ => panic!("Expected RowNotFound error"),
        }

        // Clean up
        cleanup_test_user(&pool, user1.id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_new_user_with_empty_fields() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let unique_id = Uuid::new_v4();
        let unique_sub = format!("test_google_id_{}", unique_id);

        // Test with empty name and picture
        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: format!("test_{}@gmail.com", unique_id),
            exp: 60 * 60 * 24 * 3,
            name: "".to_string(),
            picture: "".to_string(),
        };

        let user = User::create_new_user(&pool, &google_claims).await.unwrap();

        assert_eq!(user.name, "");
        assert_eq!(user.avatar, "");
        assert!(!user.private_key.is_empty());
        assert!(!user.public_key.is_empty());

        // Clean up
        cleanup_test_user(&pool, user.id).await;
        pool.close().await;
    }
}
