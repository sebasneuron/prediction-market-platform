use db_service::schema::{enums::OrderStatus, orders::Order};
use state::AppState;
use std::sync::Arc;
use utility_helpers::{log_error, log_info};

use crate::handlers::{nats_handler::handle_nats_message, ws_handler::handle_ws_messages};

mod handlers;
mod order_book;
mod state;
mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = initialize_app().await?;
    let nats_app_state = Arc::clone(&app_state);
    let ws_app_state = Arc::clone(&app_state);

    let ws_handler_join = tokio::spawn(async move {
        if let Err(e) = handle_ws_messages(ws_app_state).await {
            log_error!("Error in WS handler: {}", e);
        }
    });
    let nats_handler_join = tokio::spawn(async move {
        if let Err(e) = handle_nats_message(nats_app_state).await {
            log_error!("Error in NATS handler: {}", e);
        }
    });

    tokio::try_join!(nats_handler_join, ws_handler_join)?;

    Ok(())
}

async fn initialize_app() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState::new().await?);

    let open_orders_future = Order::get_order_by_status(&app_state.db_pool, OrderStatus::OPEN);
    let partially_updated_orders_future =
        Order::get_order_by_status(&app_state.db_pool, OrderStatus::PendingUpdate);
    let unspecified_orders_future =
        Order::get_order_by_status(&app_state.db_pool, OrderStatus::UNSPECIFIED);

    let (mut open_orders, mut partially_updated_orders, mut unspecified_orders) = tokio::try_join!(
        open_orders_future,
        partially_updated_orders_future,
        unspecified_orders_future
    )?;

    // combine open and partially updated orders
    open_orders.append(&mut partially_updated_orders);
    open_orders.append(&mut unspecified_orders);

    // synchronous block, to prevent guard from being blocked
    {
        let mut global_book = app_state.order_book.write();

        let mut order_ctn = 0;
        // iterate over open orders
        for db_order in open_orders {
            let liquidity_b = db_order.liquidity_b.clone();
            let mut order: Order = db_order.into();
            global_book.process_order(&mut order, liquidity_b);
            order_ctn += 1;
        }
        log_info!("Loaded {} open orders into the global book", order_ctn);
    }
    Ok(app_state)
}
