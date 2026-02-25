use std::sync::Arc;

use db_service::schema::{enums::OrderStatus, orders::Order};
use utility_helpers::log_warn;
use uuid::Uuid;

use crate::{
    state::AppState,
    utils::{OrderServiceError, update_services::update_service_state},
};

pub async fn cancel_order_handler(
    app_state: Arc<AppState>,
    order_id: Uuid,
) -> Result<(), OrderServiceError> {
    let order = Order::find_order_by_id(order_id, &app_state.db_pool)
        .await
        .map_err(|e| format!("Failed to find order {:#?}", e))?;

    if order.is_none() {
        log_warn!("Order with ID {} not found", order_id);
        return Ok(());
    }

    let order = order.unwrap();

    if order.status != OrderStatus::PendingCancel {
        log_warn!(
            "Order with ID {} is not in a cancellable state: {:?}",
            order_id,
            order.status
        );
        return Ok(());
    }

    // remove order from the order book
    let update_flag = {
        // sync block
        {
            let mut order_book = app_state.order_book.write();

            order_book.remove_order(
                order.market_id,
                order_id,
                order.side,
                order.outcome,
                order.price,
            )
        }
    };

    // perform db ops
    if update_flag {
        Order::update_order_status(order_id, OrderStatus::CANCELLED, &app_state.db_pool)
            .await
            .map_err(|e| format!("Failed to update order status: {:#?}", e))?;
    }

    // ws publish remaining if required...

    // update market state
    update_service_state(app_state.clone(), &order).await
}
