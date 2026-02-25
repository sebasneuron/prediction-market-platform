use axum::body::Bytes;
use prost::Message;
use proto_defs::proto_types::ws_common_types::{Channel, OperationType, WsMessage};
use utility_helpers::{log_info, log_warn, ws::types::ChannelType};
use uuid::Uuid;

use crate::{
    SafeAppState,
    core::{
        SafeSender, client_manager::SpecialKindOfClients,
        message_handlers::channel_handlers::price_posters::price_poster_handler_bin,
    },
};

// binary messages are only sended by poster channels
pub async fn handle_binary_message(
    message: &Bytes,
    client_id: &Uuid,
    tx: &SafeSender,
    state: &SafeAppState,
) {
    log_info!(
        "Received binary message from client {client_id}: {} bytes",
        message.len()
    );

    let buff: Vec<u8> = message.to_vec();

    let ws_message = WsMessage::decode(buff.as_slice());

    match ws_message {
        Ok(msg) => {
            if let Some(payload) = msg.payload {
                let type_c: OperationType = payload.ops();
                match type_c {
                    OperationType::Post => {
                        if let Some(data) = payload.data {
                            let channel = data.channel();
                            match channel {
                                Channel::Priceposter => {
                                    // we don't need to do queue based data handling just like order-book update as we know that there must be some subscriber for price poster channel when they are creating order and the payload for price update is small, while the payload for order book update can be large
                                    price_poster_handler_bin(&data, state, client_id).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    // this message send by order service to connect to the websocket server
                    OperationType::Handshake => {
                        /*
                         * This is specially used to connect order service to the websocket server and identifies the order-service's (for now) client id
                         *
                         * Example payload
                         * {
                         *  "ops": "Handshake",
                         *  "data": {
                         *     "channel": "OrderService",
                         *     "params": "shared-secret"
                         *  }
                         * }
                         */
                        if let Some(data) = payload.data {
                            let channel = data.channel();
                            match channel {
                                Channel::Orderservice => {
                                    // loading .env file to get the shared secret
                                    dotenv::dotenv().ok();
                                    let shared_secret_env = std::env::var("SHARED_SECRET")
                                        .unwrap_or_else(|_| {
                                            log_warn!("SHARED_SECRET not found in .env file");
                                            String::new()
                                        });

                                    let shared_secret = data.params;

                                    if shared_secret != shared_secret_env {
                                        log_info!(
                                            "Handshake failed for OrderService with client ID: {client_id}. Invalid shared secret."
                                        );
                                        return;
                                    }

                                    let mut client_manager_guard =
                                        state.client_manager.write().await;

                                    client_manager_guard.set_special_client(
                                        *client_id,
                                        tx.clone(),
                                        ChannelType::OrderBookPoster,
                                        SpecialKindOfClients::OrderService,
                                    );
                                    log_info!(
                                        "Handshake successful for OrderService with client ID: {client_id}"
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        log_info!("Unsupported operation type from client {client_id}: {type_c:?}");
                    }
                }
            }
        }
        Err(e) => {
            log_info!("Failed to decode binary message from client {client_id}: {e}");
        }
    }
}
