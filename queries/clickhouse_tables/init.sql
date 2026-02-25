CREATE DATABASE IF NOT EXISTS polyMarket;

USE polyMarket;

-- ############################################

-- market price data ----

-- Core table
CREATE TABLE market_price_data (
    user_id UUID,
    market_id UUID,
    yes_price Float64,
    no_price Decimal(20, 8),
    ts DateTime('UTC'),
    created_at DateTime('UTC') DEFAULT now(),
) ENGINE = MergeTree
ORDER BY ts;


-- kafka engine table
CREATE TABLE market_price_data_kafka (
    user_id UUID,
    market_id UUID,
    yes_price Decimal(20, 8),
    no_price Decimal(20, 8),
    ts String,
) ENGINE = Kafka(
    'redpanda:9092', -- broker (red panda)
    'price-updates', -- topic
    'consumer-group-price-updates', -- consumer group
    'JSONEachRow' -- format
);

-- materialized view to copy data from kafka to core table
DROP TABLE IF EXISTS market_price_data_mv;
CREATE MATERIALIZED VIEW market_price_data_mv
TO market_price_data AS
SELECT 
    user_id,
    market_id,
    yes_price,
    no_price,
    parseDateTimeBestEffort(ts) AS ts
FROM market_price_data_kafka;


-- ##################################################################################################
-- for order book data

--CORE TABLE
CREATE TABLE market_order_book (
    user_id UUID,
    market_id UUID,
    ts DateTime('UTC'),

    created_at DateTime('UTC') DEFAULT now(),

    yes_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    yes_asks Array(Tuple(price Float64, shares Float64, users UInt32)),

    no_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    no_asks Array(Tuple(price Float64, shares Float64, users UInt32))
) ENGINE = MergeTree    
ORDER BY (market_id, ts);

CREATE TABLE market_order_book_analytical (
    user_id UUID,
    market_id UUID,
    ts DateTime('UTC'),

    created_at DateTime('UTC') DEFAULT now(),

    yes_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    yes_asks Array(Tuple(price Float64, shares Float64, users UInt32)),

    no_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    no_asks Array(Tuple(price Float64, shares Float64, users UInt32))
) ENGINE = MergeTree
ORDER BY (market_id, ts);

-- KAFKA ENGINE TABLE
CREATE TABLE market_order_book_kafka (
    user_id UUID,
    market_id UUID,
    ts String,

    yes_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    yes_asks Array(Tuple(price Float64, shares Float64, users UInt32)),
    no_bids Array(Tuple(price Float64, shares Float64, users UInt32)),
    no_asks Array(Tuple(price Float64, shares Float64, users UInt32)),
) ENGINE = Kafka(
    'redpanda:9092', -- broker (red panda)
    'order-book-updates', -- topic
    'consumer-group-order-book-updates', -- consumer group
    'JSONEachRow' -- format
);

-- materialize view to copy data from kafka to core table
DROP TABLE IF EXISTS market_order_book_mv;
CREATE MATERIALIZED VIEW market_order_book_mv
TO market_order_book AS
SELECT
    user_id,
    market_id,
    parseDateTimeBestEffort(ts) AS ts,
    yes_bids,
    yes_asks,
    no_bids,
    no_asks
FROM market_order_book_kafka;

-- materialize view to copy data from kafka to analytical table
DROP TABLE IF EXISTS market_order_book_analytical_mv;
CREATE MATERIALIZED VIEW market_order_book_analytical_mv
TO market_order_book_analytical AS
SELECT
    user_id,
    market_id,
    parseDateTimeBestEffort(ts) AS ts,
    yes_bids,
    yes_asks,
    no_bids,
    no_asks
FROM market_order_book_kafka;


-- #############################################################################
-- ----------- --------- ---for volume data -----------------------

-- CORE TABLE
CREATE TABLE market_volume_data (
    user_id UUID,
    market_id UUID,
    order_id UUID,
    ts DateTime('UTC'),

    created_at DateTime('UTC') DEFAULT now(),
    side Enum8('buy' = 1, 'sell' = 2), -- buy or sell side of the order
    outcome Enum8('yes' = 1, 'no' = 2), -- yes or no outcome

    price Float64,
    quantity Float64,
    -- price * quantity is the total
    amount Decimal(20, 8) DEFAULT (price * quantity), -- total amount in USD    
) ENGINE = MergeTree
PARTITION BY toYYYYMM(ts)
ORDER BY (market_id, ts);

-- KAFKA ENGINE TABLE
CREATE TABLE market_volume_data_kafka (
    user_id UUID,
    market_id UUID,
    order_id UUID,
    ts String,
    side Enum8('buy' = 1, 'sell' = 2),
    outcome Enum8('yes' = 1, 'no' = 2),
    price Float64,
    quantity Float64
) ENGINE = Kafka(
    'redpanda:9092', -- broker (red panda)
    'volume-updates', -- topic
    'consumer-group-volume-updates', -- consumer group
    'JSONEachRow' -- format
);


-- materialized view to copy data from kafka to core table
DROP TABLE IF EXISTS market_volume_data_mv;
CREATE MATERIALIZED VIEW market_volume_data_mv
TO market_volume_data AS
SELECT
    user_id,
    market_id,
    order_id,
    parseDateTimeBestEffort(ts) AS ts,
    side,
    outcome,
    price,
    quantity
FROM market_volume_data_kafka;
