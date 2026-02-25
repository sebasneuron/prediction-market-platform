use std::{collections::HashSet, sync::Arc};

use async_nats::connect;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use parking_lot::RwLock;
use prost::Message;
use proto_defs::proto_types::ws_common_types::{
    Channel, OperationType, Payload, WsData, WsMessage,
};
use rdkafka::{ClientConfig, producer::FutureProducer};
use tokio::{net::TcpStream, sync::RwLock as AsyncRwLock};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message as WsMessageSentType,
    tungstenite::client::IntoClientRequest,
};
use utility_helpers::{log_error, log_info, types::EnvVarConfig};
use uuid::Uuid;

use crate::order_book::global_book::GlobalMarketBook;

pub struct AppState {
    // async states
    pub db_pool: sqlx::PgPool,
    pub jetstream: async_nats::jetstream::Context, // it's already thread safe for async operations internally...
    pub producer: AsyncRwLock<FutureProducer>,
    pub ws_tx:
        AsyncRwLock<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessageSentType>>,
    pub ws_rx: AsyncRwLock<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,

    // sync states
    // preferring RwLock rather than tokio's rwLock because the operations on orderbook are not async (to gain maximum performance)
    pub order_book: Arc<RwLock<GlobalMarketBook>>,
    pub market_subs: Arc<RwLock<HashSet<Uuid>>>, // market id subscribers (for order book updates)
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let nc = connect(&env_var_config.nc_url)
            .await
            .expect("Failed to connect to NATS server");
        log_info!("Connected to NATS");

        let jetstream = async_nats::jetstream::new(nc);
        let db_pool = sqlx::PgPool::connect(&env_var_config.database_url)
            .await
            .expect("Failed to connect to the database");
        log_info!("Connected to database");

        let producer = ClientConfig::new()
            .set("bootstrap.servers", &env_var_config.kafka_url)
            .create::<FutureProducer>()
            .expect("Failed to create Kafka producer");

        log_info!("Connected to red panda (kafka)");

        let websocket_req = format!("{}/ws", env_var_config.websocket_url)
            .into_client_request()
            .expect("Failed to create WebSocket request");

        let (mut stream, _) = connect_async(websocket_req)
            .await
            .expect("Failed to connect to WebSocket server");
        log_info!("Connected to WebSocket server");

        // handshaking with ws server
        let message = WsMessage {
            id: None,
            payload: Some(Payload {
                ops: OperationType::Handshake as i32,
                data: Some(WsData {
                    channel: Channel::Orderservice as i32,
                    params: env_var_config.shared_secret,
                }),
            }),
        };

        let bin_data = message.encode_to_vec();

        if let Err(e) = stream
            .send(WsMessageSentType::Binary(bin_data.into()))
            .await
        {
            log_error!("Handshake failed with websocket server {e}");
        }
        log_info!("Handshake complete with websocket service");
        let (tx, rx) = stream.split();

        let order_book = Arc::new(RwLock::new(GlobalMarketBook::new()));
        let market_subs = Arc::new(RwLock::new(HashSet::new()));

        Ok(AppState {
            db_pool,
            jetstream,
            producer: AsyncRwLock::new(producer),
            ws_rx: AsyncRwLock::new(rx),
            ws_tx: AsyncRwLock::new(tx),
            order_book,
            market_subs,
        })
    }
}
