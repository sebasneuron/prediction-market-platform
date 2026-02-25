pub enum KafkaTopics {
    PriceUpdates,
    MarketOrderBookUpdate,
    VolumeUpdates,
}

impl KafkaTopics {
    pub fn from_str(topic: &str) -> Option<Self> {
        if topic == "order-book-updates" {
            Some(KafkaTopics::MarketOrderBookUpdate)
        } else if topic == "price-updates" {
            Some(KafkaTopics::PriceUpdates)
        } else if topic == "volume-updates" {
            Some(KafkaTopics::VolumeUpdates)
        } else {
            None
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            KafkaTopics::PriceUpdates => "price-updates",
            KafkaTopics::MarketOrderBookUpdate => "order-book-updates",
            KafkaTopics::VolumeUpdates => "volume-updates",
        }
    }
}
