use std::sync::Arc;

use grpc_service::{
    generated::{
        markets::market_service_server::MarketServiceServer,
        price::price_service_server::PriceServiceServer,
    },
    procedures::{market_services::MarketServiceStub, price_services::PriceServiceStub},
    state::AppState,
};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;
use utility_helpers::log_info;

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/descriptor.bin");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // enabling tracer
    tracing_subscriber::fmt::init();

    let reflector_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .include_reflection_service(true)
        .build_v1alpha()?;

    let app_state = AppState::new().await?;
    let state = Arc::new(app_state);

    // services
    let market_service_layer = MarketServiceStub {
        state: state.clone(),
    };
    let pair_service_layer = PriceServiceStub {
        state: state.clone(),
    };

    log_info!("GRPC server running on port grpc://localhost:5010");

    let addr = "0.0.0.0:5010".parse()?;

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive()) // allows CORS for all origins
        .layer(GrpcWebLayer::new())
        .add_service(reflector_service)
        .add_service(MarketServiceServer::new(market_service_layer))
        .add_service(PriceServiceServer::new(pair_service_layer))
        .serve(addr)
        .await?;

    Ok(())
}
