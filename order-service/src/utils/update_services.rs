/*
 * This file is ued to update the state of services
 *
 * - Pushes the price into clickhouse via kafka
 * - Send the data to the websocket which serves the users
 * - Publishes the data to NATS for other services to consume
 */

use std::{str::FromStr, sync::Arc, time::Duration};

use db_service::schema::{
    enums::{OrderStatus, Outcome},
    orders::Order,
};
use futures_util::SinkExt;
use prost::Message;
use proto_defs::proto_types::ws_common_types::{
    Channel, OperationType, Payload, WsData, WsMessage,
};
use rdkafka::producer::FutureRecord;
use rust_decimal::Decimal;
use tokio_tungstenite::tungstenite::Message as WsMessageType;
use utility_helpers::{
    kafka_topics::KafkaTopics,
    log_error, log_info,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::OrderBookUpdateData},
    types::OrderBookDataStruct,
};

use crate::{state::AppState, utils::OrderServiceError};

pub async fn update_service_state(
    app_state: Arc<AppState>,
    order: &Order,
) -> Result<(), OrderServiceError> {
    // variable declarations....
    let current_time = chrono::Utc::now();
    let market_id = order.market_id;

    let producer_lock_future = app_state.producer.read();
    let ws_publisher_lock_future = app_state.ws_tx.write();

    let (producer, mut ws_publisher) = tokio::join!(producer_lock_future, ws_publisher_lock_future);

    let js_guard = app_state.jetstream.clone();

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////// Sync code block star ///////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////

    // market id validation and current market state
    let (yes_price, no_price, yes_orders_data, no_orders_data, required_market_subs) = {
        // sync block
        {
            let order_book = app_state.order_book.read();

            let yes_price = order_book
                .get_market_price(&market_id, Outcome::YES)
                .unwrap_or_else(|| Decimal::new(5, 1));
            let no_price = order_book
                .get_market_price(&market_id, Outcome::NO)
                .unwrap_or_else(|| Decimal::new(5, 1));

            let yes_orders = order_book.get_orders(&market_id, Outcome::YES);
            let no_orders = order_book.get_orders(&market_id, Outcome::NO);

            // processing yes orders
            let yes_orders_data = if let Some(yes_orders) = yes_orders {
                yes_orders.get_order_book()
            } else {
                OrderBookDataStruct::default()
            };
            // processing no orders
            let no_orders_data = if let Some(no_orders) = no_orders {
                no_orders.get_order_book()
            } else {
                OrderBookDataStruct::default()
            };
            let market_subs_guard = app_state.market_subs.read();
            let required_market_subs = market_subs_guard.contains(&market_id);

            (
                // passing states from sync codeblock to async code block....
                yes_price,
                no_price,
                yes_orders_data,
                no_orders_data,
                required_market_subs,
            )
        }
    };

    log_info!(
        "Order processed.. YES Price: {}, NO Price: {}",
        yes_price,
        no_price
    );

    //////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////// Sync code block end ///////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////// kafka preparation //////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////////

    let ts = current_time.to_rfc3339();
    let data_to_publish_for_price_update = serde_json::json!({
        "user_id": order.user_id.to_string(),
        "market_id": market_id.to_string(),
        "yes_price":yes_price.to_string(),
        "no_price": no_price.to_string(),
        "ts": ts,
    })
    .to_string();

    let data_to_publish_for_order_book_update = serde_json::json!({
        "user_id": order.user_id.to_string(),
        "market_id": market_id.to_string(),
        "yes_asks": yes_orders_data.asks,
        "yes_bids": yes_orders_data.bids,
        "no_asks": no_orders_data.asks,
        "no_bids": no_orders_data.bids,
        "ts": ts,
    })
    .to_string();

    let data_to_publish_for_volume_update = serde_json::json!({
        "user_id": order.user_id.to_string(),
        "market_id": market_id,
        "order_id": order.id,
        "ts": ts,
        "side": order.side,
        "outcome": order.outcome,
        "price": order.price,
        "quantity": order.quantity,
    })
    .to_string();

    let price_update_topic = KafkaTopics::PriceUpdates.to_string();
    let market_order_book_update_topic = KafkaTopics::MarketOrderBookUpdate.to_string();
    let volume_update_topic = KafkaTopics::VolumeUpdates.to_string();

    let market_id_str = market_id.to_string();

    let mut kafka_futures = vec![];
    let mut nats_futures = vec![];

    let record_price_update = FutureRecord::to(price_update_topic)
        .payload(&data_to_publish_for_price_update)
        .key(&market_id_str);
    let record_order_book_update = FutureRecord::to(market_order_book_update_topic)
        .payload(&data_to_publish_for_order_book_update)
        .key(&market_id_str);
    let record_volume_update = FutureRecord::to(volume_update_topic)
        .payload(&data_to_publish_for_volume_update)
        .key(&market_id_str);

    let send_producer_future_price = producer.send(record_price_update, Duration::from_secs(0));
    let send_producer_future_order_book =
        producer.send(record_order_book_update, Duration::from_secs(0));

    kafka_futures.push(send_producer_future_price);
    kafka_futures.push(send_producer_future_order_book);

    if order.status == OrderStatus::FILLED {
        let send_producer_future_volume =
            producer.send(record_volume_update, Duration::from_secs(0));
        kafka_futures.push(send_producer_future_volume);
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////// NATS processing //////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////////

    let combined_data = OrderBookUpdateData {
        yes_book: yes_orders_data.clone(),
        no_book: no_orders_data.clone(),
        market_id: market_id,
        timestamp: current_time.to_rfc3339(),
    };

    if required_market_subs {
        let message_pack_encoded = serialize_to_message_pack(&combined_data)?;

        // pushing message to queue
        let subject = NatsSubjects::MarketBookUpdate(market_id).to_string();
        let publish_msg_future = js_guard.publish(subject, message_pack_encoded.into());

        nats_futures.push(publish_msg_future);
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////// sending message to websocket ////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////

    let yes_price = f64::from_str(&yes_price.to_string())
        .map_err(|_| "Failed to parse yes price to f64".to_string())?;
    let no_price = f64::from_str(&no_price.to_string())
        .map_err(|_| "Failed to parse no price to f64".to_string())?;

    let market_data = serde_json::json!({
        "market_id": market_id,
        "yes_price": yes_price,
        "no_price": no_price,
        "timestamp": current_time.timestamp_millis(),
    })
    .to_string();

    let message = WsMessage {
        id: None,
        payload: Some(Payload {
            ops: OperationType::Post as i32,
            data: Some(WsData {
                channel: Channel::Priceposter as i32,
                params: market_data,
            }),
        }),
    };

    let bin_data = message.encode_to_vec();

    let ws_broadcast_future = ws_publisher.send(WsMessageType::Binary(bin_data.into()));

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////////////////////////

    let result = tokio::join!(
        futures_util::future::join_all(kafka_futures),
        futures_util::future::join_all(nats_futures),
        ws_broadcast_future
    );
    for res in result.0 {
        match res {
            Ok(_) => log_info!("Kafka message sent successfully"),
            Err(e) => log_error!("Failed to send Kafka message: {:#?}", e),
        }
    }
    for res in result.1 {
        match res {
            Ok(_) => log_info!("NATS message sent successfully"),
            Err(e) => log_error!("Failed to send NATS message: {:#?}", e),
        }
    }

    match result.2 {
        Ok(_) => log_info!("WebSocket message sent successfully"),
        Err(e) => log_error!("Failed to send WebSocket message: {:#?}", e),
    }

    Ok(())
}
