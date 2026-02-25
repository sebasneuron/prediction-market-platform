use std::sync::Arc;

use async_nats::jetstream;
use db_service::schema::orders::Order;
use futures_util::StreamExt;
use utility_helpers::{
    log_error, log_info,
    message_pack_helper::deserialize_from_message_pack,
    nats_helper::{
        NatsSubjects,
        types::{InitializeOrderBookMessage, MarketOrderCreateMessage, UpdateOrderMessage},
    },
};

use crate::{
    handlers::nats_handler::{
        add_order_handler::add_order_handler, cancel_order_handler::cancel_order_handler,
        create_order_handler::create_order_handler, update_order_handler::update_order_handler,
    },
    state::AppState,
    utils::OrderServiceError,
};

pub mod add_order_handler;
pub mod cancel_order_handler;
pub mod create_order_handler;
pub mod update_order_handler;

pub async fn handle_nats_message(app_state: Arc<AppState>) -> Result<(), OrderServiceError> {
    let stream_guard = app_state.jetstream.clone();
    let stream = stream_guard
        .get_or_create_stream(jetstream::stream::Config {
            // these `ORDER` name does not indicate the operations on orders, instead it indicates that the streams is used by order-service microservice, so don't confuse it with the order name and same for it's topics, all topics are prefixed with `order.`
            name: "ORDER".to_string(),
            subjects: vec!["order.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("order_os".to_string()),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(Ok(message)) = messages.next().await {
        let subject = message.subject.clone();
        let subject_str = subject.as_str();
        let subject = NatsSubjects::from_string(subject_str)
            .ok_or_else(|| format!("Invalid subject: {}", subject))?;

        match subject {
            NatsSubjects::OrderCreate => {
                let order_id = String::from_utf8(message.payload.to_vec())
                    .map_err(|_| "Failed to convert payload to string".to_string())?;
                log_info!("Received order ID: {}", order_id);
                let order_id = uuid::Uuid::parse_str(&order_id)
                    .map_err(|_| "Failed to parse order ID from string".to_string())?;
                let _ = create_order_handler(app_state.clone(), order_id, None)
                    .await
                    .map_err(|e| {
                        log_error!("Error occur while adding order in book {e}");
                    });
            }
            NatsSubjects::OrderCancel => {
                let order_id = String::from_utf8(message.payload.to_vec())
                    .map_err(|_| "Failed to convert payload to string".to_string())?;

                let order_id = uuid::Uuid::parse_str(&order_id)
                    .map_err(|_| "Failed to parse order ID from string".to_string())?;
                let _ = cancel_order_handler(app_state.clone(), order_id)
                    .await
                    .map_err(|e| {
                        log_error!("Error occur while cancelling order {e}");
                    });
            }
            NatsSubjects::OrderUpdate => {
                let serialized_message = message.payload.to_vec();
                let deserialized_message = deserialize_from_message_pack::<UpdateOrderMessage>(
                    &serialized_message.as_slice(),
                )?;

                let _ = update_order_handler(app_state.clone(), deserialized_message)
                    .await
                    .map_err(|e| {
                        log_error!("Error occur while updating order {e}");
                    });
            }
            NatsSubjects::MarketOrderCreate => {
                let serialized_message = message.payload.to_vec();
                let deserialized_message =
                    deserialize_from_message_pack::<MarketOrderCreateMessage>(&serialized_message)?;

                let _ = create_order_handler(
                    app_state.clone(),
                    deserialized_message.order_id,
                    Some(deserialized_message.budget),
                )
                .await
                .map_err(|e| {
                    log_error!("Error occur while adding market order in book {e}");
                });
            }
            NatsSubjects::InitializeOrderBook => {
                let serialized_message = message.payload.to_vec();
                let deserialized_message = deserialize_from_message_pack::<
                    InitializeOrderBookMessage<Order>,
                >(&serialized_message)?;

                let _ = add_order_handler(
                    app_state.clone(),
                    &deserialized_message.orders,
                    deserialized_message.liquidity_b,
                )
                .await
                .map_err(|e| {
                    log_error!("Error occur while initializing order book {e}");
                    e
                })?;
            }
            NatsSubjects::FinalizeMarket => {
                let market_id = String::from_utf8(message.payload.to_vec())
                    .map_err(|_| "Failed to convert payload to string".to_string())?;
                let market_id = uuid::Uuid::parse_str(&market_id)
                    .map_err(|_| "Failed to parse market ID from string".to_string())?;

                let mut global_book_guard = app_state.order_book.write();
                if global_book_guard.remove_market(&market_id) {
                    log_info!("Market with ID {} removed from global book", market_id);
                } else {
                    log_error!(
                        "Failed to remove market with ID {} from global book",
                        market_id
                    );
                }
            }
            _ => {}
        }

        // sending ack in either case...
        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}
