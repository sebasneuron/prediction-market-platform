use axum::extract::ws::{Message, Utf8Bytes};
use serde_json::json;
use utility_helpers::{
    log_error, log_warn,
    ws::types::{ChannelType, ClientMessage, MessagePayload},
};
use uuid::Uuid;

use crate::{
    SafeAppState,
    core::{SafeSender, client_manager::SpecialKindOfClients, send_message},
};

pub async fn handle_text_message(
    message: &Utf8Bytes,
    client_id: &Uuid,
    tx: &SafeSender,
    state: &SafeAppState,
) {
    match serde_json::from_str::<ClientMessage>(message) {
        Ok(client_message) => match client_message.payload {
            MessagePayload::Subscribe { channel } => {
                let deserialized_channel = ChannelType::from_str(&channel);

                let channel_type = match deserialized_channel {
                    Some(channel_type) => channel_type,
                    None => {
                        log_error!("Invalid channel type from client {client_id}: {channel}");
                        if let Err(e) = send_message(
                            tx,
                            Message::Text(format!("Invalid channel {channel}").into()),
                        )
                        .await
                        {
                            log_error!("Failed to send error response to client {client_id}: {e}");
                        }
                        return;
                    }
                };

                let mut channel_manager_guard = state.client_manager.write().await;
                match channel_type {
                    ChannelType::OrderBookUpdate(_) => {
                        // sending payload to special client
                        let payload = json!({
                            "payload": {
                                "type": "Subscribe",
                                "data":{
                                    "channel": channel_type.to_str() // market id is in channel...
                                }
                            }
                        })
                        .to_string();

                        // special client tx
                        let tx = channel_manager_guard
                            .get_special_client(SpecialKindOfClients::OrderService);
                        if let Some(tx) = tx {
                            if let Err(e) = send_message(&tx, Message::Text(payload.into())).await {
                                log_error!("Failed to send message to special service {e}");
                            }
                        } else {
                            log_warn!("No special service client found in ws server");
                        }
                    }
                    _ => {}
                }

                channel_manager_guard.add_client(channel_type, *client_id, tx.clone());

                let message = json!({
                    "type": "subscribed",
                    "channel": channel,
                })
                .to_string();
                if let Err(e) = send_message(tx, message.into()).await {
                    log_error!(
                        "Failed to send subscription confirmation to client {client_id}: {e}"
                    );
                }
            }
            MessagePayload::Unsubscribe { channel } => {
                let deserialized_channel = ChannelType::from_str(&channel);

                let channel_type = match deserialized_channel {
                    Some(channel_type) => channel_type,
                    None => {
                        log_error!("Invalid channel type from client {client_id}: {channel}");
                        if let Err(e) =
                            send_message(tx, Message::Text("Invalid channel".into())).await
                        {
                            log_error!("Failed to send error response to client {client_id}: {e}");
                        }
                        return;
                    }
                };

                let mut channel_manager_guard = state.client_manager.write().await;
                channel_manager_guard.remove_client(&channel_type, &client_id);
                let message = json!({
                    "type": "unsubscribed",
                    "channel": channel
                })
                .to_string();
                if let Err(e) = send_message(tx, message.into()).await {
                    log_error!(
                        "Failed to send unsubscription confirmation to client {client_id}: {e}"
                    );
                }
            }
        },
        Err(e) => {
            log_error!("Failed to parse ClientMessage from client {client_id}: {e}");
            if let Err(e) = send_message(tx, Message::Text("Invalid message format".into())).await {
                log_error!("Failed to send error response to client {client_id}: {e}");
            }
        }
    }
}
