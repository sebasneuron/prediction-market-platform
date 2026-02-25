use async_nats::jetstream;
use futures::StreamExt;
use utility_helpers::{
    log_info,
    message_pack_helper::deserialize_from_message_pack,
    nats_helper::{NatsSubjects, types::OrderBookUpdateData},
};

use crate::{SafeAppState, nats_handler::handle_market_book_update::handle_market_book_update};

pub mod handle_market_book_update;

pub async fn nats_handler(state: SafeAppState) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("NATS handler started for order book service");

    let stream_guard = state.jetstream.clone();

    let stream = stream_guard
        .get_or_create_stream(jetstream::stream::Config {
            name: "ORDER".to_string(),
            subjects: vec!["order.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("order_ws".to_string()),
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
            NatsSubjects::MarketBookUpdate(market_id) => {
                log_info!("Received market book update for market ID: {}", market_id);
                let data_buff = message.payload.to_vec();
                let data =
                    deserialize_from_message_pack::<OrderBookUpdateData>(&data_buff.as_slice())?;
                let market_book_handler_state = state.clone();
                handle_market_book_update(market_book_handler_state, data).await?;
            }
            _ => {
                log_info!("Received message on unsupported subject: {}", subject);
            }
        }
        // Acknowledge the message
        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}
