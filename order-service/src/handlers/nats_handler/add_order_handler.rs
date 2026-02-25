use std::sync::Arc;

use db_service::schema::orders::Order;
use rust_decimal::Decimal;
use utility_helpers::log_error;

use crate::{
    state::AppState,
    utils::{OrderServiceError, update_services::update_service_state},
};

pub async fn add_order_handler(
    state: Arc<AppState>,
    orders: &Vec<Order>,
    liquidity_b: Decimal,
) -> Result<(), OrderServiceError> {
    // synchronous write lock to the order book
    {
        let mut order_book_guard = state.order_book.write();

        for order in orders.iter() {
            order_book_guard.add_order(order, liquidity_b);
        }
    }

    // asynchronous service state update
    for order in orders.iter() {
        update_service_state(state.clone(), order)
            .await
            .map_err(|e| {
                log_error!(
                    "Failed to update service state for order {}: {}",
                    order.id,
                    e
                );
                format!(
                    "Failed to update service state for order {}: {}",
                    order.id, e
                )
            })?;
    }
    Ok(())
}
