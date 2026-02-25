use axum::{
    Router,
    extract::{State, ws::WebSocketUpgrade},
    routing::any,
};
use std::sync::Arc;
use tracing_subscriber;
use utility_helpers::{log_error, log_info};

use crate::{
    core::handle_connection::handle_connection, nats_handler::nats_handler,
    state::WebSocketAppState,
};

mod core;
mod nats_handler;
mod state;

pub type SafeAppState = Arc<WebSocketAppState>;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let app_state = WebSocketAppState::new().await?;
    let app_state = Arc::new(app_state);
    let nats_handler_state = app_state.clone();

    let app = Router::new()
        .route("/", any(|| async { "Hello from WebSocket server!" }))
        .route("/ws", any(socket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("[::]:4010").await?;

    log_info!("Starting WebSocket server on port 4010");
    tokio::spawn(async move {
        if let Err(e) = nats_handler(nats_handler_state).await {
            log_error!("Error in NATS handler: {}", e);
            panic!("NATS handler encountered an error");
        };
    });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SafeAppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}
