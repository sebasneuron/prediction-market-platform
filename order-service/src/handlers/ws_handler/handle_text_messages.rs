use std::sync::Arc;

use utility_helpers::{
    log_info, log_warn,
    ws::types::{ChannelType, ClientMessage, MessagePayload},
};

use crate::state::AppState;

pub(super) async fn handle_text_messages(state: &Arc<AppState>, message: &ClientMessage) {
    match &message.payload {
        MessagePayload::Subscribe { channel } => {
            let channel = ChannelType::from_str(&channel);
            if let Some(channel_type) = channel {
                match channel_type {
                    ChannelType::OrderBookUpdate(market_id) => {
                        /*
                           Example payload
                           {
                               "payload": {
                                   "type": "Subscribe",
                                   "data": {
                                       "channel": "order_book_update:<market_id>"
                                   }
                               }
                           }
                        */
                        log_info!("Subscribing to order book updates for market: {market_id}");
                        {
                            let mut market_subs = state.market_subs.write();
                            market_subs.insert(market_id);
                        }

                        log_info!("Subscribed to order book updates for market: {market_id}");
                    }
                    _ => {
                        log_warn!(
                            "Unsupported channel type for subscription: {:?}",
                            channel_type
                        );
                    }
                }
            }
        }
        MessagePayload::Unsubscribe { channel } => {
            let channel = ChannelType::from_str(&channel);
            if let Some(channel_type) = channel {
                match channel_type {
                    ChannelType::OrderBookUpdate(market_id) => {
                        log_info!("Unsubscribing from order book updates for market: {market_id}");
                        let mut market_subs = state.market_subs.write();
                        market_subs.remove(&market_id);
                    }
                    _ => {
                        log_warn!("Unsupported channel type for unsubscription: {channel_type:?}");
                    }
                }
            }
        }
    }
}
