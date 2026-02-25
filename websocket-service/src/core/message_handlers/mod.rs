use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, stream::SplitStream};
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::{
    SafeAppState,
    core::{
        SafeSender,
        message_handlers::{
            handle_binary_message::handle_binary_message, handle_text_message::handle_text_message,
        },
        send_message,
    },
};

pub mod channel_handlers;
pub mod handle_binary_message;
pub mod handle_text_message;

pub async fn handle_message(
    rx: &mut SplitStream<WebSocket>,
    tx: &SafeSender,
    client_id: &Uuid,
    state: &SafeAppState,
) {
    while let Some(message) = rx.next().await {
        match message {
            Ok(message) => match message {
                Message::Text(text) => {
                    handle_text_message(&text, client_id, tx, state).await;
                }
                Message::Binary(bin) => {
                    // protobuf
                    handle_binary_message(&bin, client_id, tx, state).await;
                }
                Message::Pong(_) => {
                    log_info!("Received Pong from client {client_id}");
                }
                Message::Ping(_) => {
                    log_info!("Received Ping from client {client_id}");
                    if let Err(e) = send_message(tx, Message::Pong(vec![].into())).await {
                        log_error!("Failed to send Pong to client {client_id}: {e}");
                    }
                }
                Message::Close(_) => {
                    log_info!("Client {client_id} disconnected");
                    return;
                }
            },
            Err(e) => {
                log_error!("Error receiving message from client {client_id}: {e}");
            }
        }
    }
}
