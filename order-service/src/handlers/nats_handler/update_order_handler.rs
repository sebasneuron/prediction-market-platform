use std::sync::Arc;

use db_service::schema::{enums::OrderStatus, orders::Order};
use utility_helpers::{log_error, nats_helper::types::UpdateOrderMessage};

use crate::{
    state::AppState,
    utils::{
        OrderServiceError, update_matched_orders::update_matched_orders,
        update_services::update_service_state,
    },
};

pub async fn update_order_handler(
    app_state: Arc<AppState>,
    data: UpdateOrderMessage,
) -> Result<(), OrderServiceError> {
    let order = Order::find_order_by_id(data.order_id, &app_state.db_pool)
        .await
        .map_err(|e| {
            log_error!("Error finding order: {}", e);
            e
        })?;

    if order.is_none() {
        log_error!("Order not found with ID: {}", data.order_id);
        return Err("Order not found".into());
    }

    let mut order = order.unwrap();

    if order.status != OrderStatus::PendingUpdate {
        log_error!(
            "Order with ID {} is not in a updatable state: {:?}",
            data.order_id,
            order.status
        );
        return Ok(());
    }

    // sync block
    let matches = {
        let mut order_book = app_state.order_book.write();

        let flg = order_book.update_order(&mut order, data.new_price, data.new_quantity);

        if flg {
            order_book.process_order_without_liquidity(&mut order)
        } else {
            Vec::new()
        }
    };

    order
        .update(&app_state.db_pool)
        .await
        .map_err(|e| format!("Failed to update order: {e:#?}"))?;

    tokio::try_join!(
        update_matched_orders(matches, app_state.clone(), &order),
        update_service_state(app_state.clone(), &order)
    )?;

    Ok(())
}
