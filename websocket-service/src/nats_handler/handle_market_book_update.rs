use axum::extract::ws::Message as WsMessage;
use prost::Message;
use utility_helpers::{log_info, nats_helper::types::OrderBookUpdateData, ws::types::ChannelType};

use crate::{SafeAppState, core::send_message};

pub async fn handle_market_book_update(
    state: SafeAppState,
    data: OrderBookUpdateData,
) -> Result<(), Box<dyn std::error::Error>> {
    let client_manager_guard = state.client_manager.read().await;
    let market_id = data.market_id;

    let subscribers_opt =
        client_manager_guard.get_clients(&ChannelType::OrderBookUpdate(market_id));
    let message = data.get_prost_market_book(market_id);
    let message = message.encode_to_vec();

    if let Some(subscribers) = subscribers_opt {
        for (_, tx) in subscribers.iter() {
            if let Err(e) = send_message(tx, WsMessage::Binary(message.clone().into())).await {
                log_info!("Failed to send market book update to client: {}", e);
            } else {
                log_info!(
                    "Market book update sent to client for market ID: {}",
                    market_id
                );
            }
        }
    }

    Ok(())
}
