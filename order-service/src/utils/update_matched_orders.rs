use std::sync::Arc;

use db_service::schema::{
    enums::{OrderSide, OrderStatus},
    orders::Order,
    user_holdings::UserHoldings,
    user_trades::UserTrades,
    users::User,
};
use rust_decimal::Decimal;

use crate::{
    order_book::outcome_book::OrderBookMatchedOutput, state::AppState, utils::OrderServiceError,
};

pub async fn update_matched_orders(
    matched_order: Vec<OrderBookMatchedOutput>,
    app_state: Arc<AppState>,
    order: &Order,
) -> Result<(), OrderServiceError> {
    for match_item in matched_order {
        // update the opposite order's filled quantity
        let current_order_id = match_item.order_id;
        let opposite_order_id = match_item.opposite_order_id;
        let quantity = match_item.matched_quantity;
        let opposite_order_new_status = if match_item.opposite_order_filled_quantity
            == match_item.opposite_order_total_quantity
        {
            OrderStatus::FILLED
        } else {
            OrderStatus::OPEN
        };

        // current order is updated previously, as it's get mutated from order matching engine
        Order::update_order_status_and_filled_quantity(
            &app_state.db_pool,
            opposite_order_id,
            opposite_order_new_status,
            match_item.opposite_order_filled_quantity,
        )
        .await
        .map_err(|e| format!("Failed to update opposite order: {:#?}", e))?;

        // fetching user ids of current and opposite orders for updating user trades and holdings
        let get_current_order_user_id_future =
            Order::get_order_user_id(&app_state.db_pool, current_order_id);
        let get_opposite_order_user_id_future =
            Order::get_order_user_id(&app_state.db_pool, opposite_order_id);

        // we want both id, so try_join instead of join
        let (current_order_user_id, opposite_order_user_id) = tokio::try_join!(
            get_current_order_user_id_future,
            get_opposite_order_user_id_future
        )
        .map_err(|e| {
            format!(
                "Failed to get user ids for current order {:#?} and opposite order {:#?}: {:#?}",
                current_order_id, opposite_order_id, e
            )
        })?;

        /////// Database Transaction start ////////

        // here we are preferring to use db transaction instead of rust's parallel (tokio::join) operation processing (it compromises performance and perform sequential processing), +we can't share `tx` across async tasks parallelly
        let mut tx = app_state.db_pool.begin().await?;

        let (current_order_type, opposite_order_type) = match order.side {
            OrderSide::BUY => (OrderSide::SELL, OrderSide::BUY), // if my current order is buy, then trade type is sell for opposite order as I want to sell my shares
            OrderSide::SELL => (OrderSide::BUY, OrderSide::SELL), // if my current order is sell, then trade type is buy for opposite order as I want to buy shares
        };

        // create current order's user trade
        UserTrades::create_user_trade(
            &mut *tx,
            current_order_id,
            opposite_order_id,
            order.user_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
            current_order_type,
        )
        .await
        .map_err(|e| format!("Failed to create user trade: {:#?}", e))?;

        // create opposite order's user trade
        UserTrades::create_user_trade(
            &mut *tx,
            current_order_id,
            opposite_order_id,
            current_order_user_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
            opposite_order_type,
        )
        .await?;

        let (current_order_user_updated_holding, opposite_order_user_updated_holdings) =
            match order.side {
                OrderSide::BUY => (quantity, -quantity),
                OrderSide::SELL => (-quantity, quantity),
            };

        UserHoldings::update_user_holdings(
            &mut *tx,
            current_order_user_id,
            order.market_id,
            current_order_user_updated_holding,
            order.outcome,
        )
        .await?;

        UserHoldings::update_user_holdings(
            &mut *tx,
            opposite_order_user_id,
            order.market_id,
            opposite_order_user_updated_holdings,
            order.outcome,
        )
        .await?;

        // updating user balances
        User::update_two_users_balance(
            &mut *tx,
            current_order_user_id,
            opposite_order_user_id,
            (match_item.matched_quantity * match_item.price) * Decimal::from(100),
            order.side,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {:#?}", e))?;

        /////// Database Transaction end ////////
    }

    Ok(())
}
