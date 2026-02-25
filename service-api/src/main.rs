use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use state::AppState;
use utility_helpers::log_info;

mod routes;
mod state;
mod utils;

pub mod bloom_f;

const PORT: u16 = 8080;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = format!("[::]:{}", PORT);

    let state = AppState::new().await?;
    state.run_migrations().await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log_info!("service-api is listening on http://localhost:{}", PORT);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
