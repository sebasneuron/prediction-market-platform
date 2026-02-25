use rust_decimal::Decimal;
use uuid::Uuid;

pub struct Procedures;

impl Procedures {
    pub async fn call_update_order_and_process_trade(
        pool: &sqlx::PgPool,
        current_order_id: Uuid,
        opposite_order_id: Uuid,
        new_filled_quantity: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "CALL polymarket.update_order_and_process_trade($1, $2, $3);",
            current_order_id,
            opposite_order_id,
            new_filled_quantity
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use std::str::FromStr;

//     use rust_decimal_macros::dec;
//     use utility_helpers::types::GoogleClaims;

//     use crate::schema::{
//         enums::{OrderSide, Outcome},
//         market::Market,
//         orders::Order,
//         users::User,
//     };

//     use super::*;

//     async fn get_db_pool() -> sqlx::PgPool {
//         dotenv::dotenv().ok();
//         let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//         sqlx::PgPool::connect(&database_url).await.unwrap()
//     }

//     #[tokio::test]
//     async fn test_call_update_order_and_process_trade() {
//         let pool = get_db_pool().await;
//         let user_id = Uuid::from_str("593c08f0-6695-4b42-86f1-546e5553011c").unwrap();
//         let other_user_id = Uuid::from_str("4f2f5832-f83e-4c25-aa14-d62efc52e912").unwrap();
//         let test_market = Market::create_new_market(
//             "admin".to_string(),
//             "admin".to_string(),
//             "admin".to_string(),
//             dec!(100),
//             &pool,
//         )
//         .await
//         .unwrap();

//         let buy_order = Order::create_order(
//             user_id,
//             test_market.id,
//             dec!(0.3),
//             dec!(10),
//             OrderSide::BUY,
//             Outcome::YES,
//             &pool,
//         )
//         .await
//         .unwrap();
//         let sell_order = Order::create_order(
//             other_user_id,
//             test_market.id,
//             dec!(0.3),
//             dec!(10),
//             OrderSide::SELL,
//             Outcome::YES,
//             &pool,
//         )
//         .await
//         .unwrap();

//         // Call the procedure to update order and process trade
//         // let result = Procedures::call_update_order_and_process_trade(
//         //     &pool,
//         //     buy_order.id,
//         //     sell_order.id,
//         //     dec!(5),
//         // )
//         // .await;
//         // println!("Result: {:?}", result);
//         // cleanup

//         sqlx::query!("DELETE FROM polymarket.orders WHERE id = $1;", buy_order.id)
//             .execute(&pool)
//             .await
//             .unwrap();
//         sqlx::query!(
//             "DELETE FROM polymarket.markets WHERE id = $1;",
//             test_market.id
//         )
//         .execute(&pool)
//         .await
//         .unwrap();
//     }
// }
