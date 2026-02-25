use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use tokio::sync::Mutex;

pub mod client_manager;
pub mod handle_connection;
pub mod message_handlers;

// mutex because rx.next() method requires mutable access, so one reader and writer at a time...
pub type SafeSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub(super) async fn send_message(tx: &SafeSender, message: Message) -> Result<(), axum::Error> {
    tx.lock().await.send(message).await
}
