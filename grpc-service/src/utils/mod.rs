pub mod clickhouse_schema;
pub mod macros;
pub mod timeframe;

pub mod clickhouse_queries {
    pub const ORDER_BOOK_INITIALS: &str = r#"
            SELECT
                market_id,
                ts,
                created_at,

                CAST(arraySlice(
                    arrayFilter(x -> x.2 > 0, yes_bids), 1, ?
                    ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_bids,
                CAST(arraySlice(
                    arrayFilter(x -> x.2 > 0, yes_asks), 1, ?
                    ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_asks,
                CAST(arraySlice(
                    arrayFilter(x -> x.2 > 0, no_bids), 1, ?
                    ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_bids,
                CAST(arraySlice(
                    arrayFilter(x -> x.2 > 0, no_asks), 1, ?
                    ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_asks
            FROM market_order_book WHERE market_id = ?
            ORDER BY ts DESC
            LIMIT 1
        "#;

    pub const MARKET_PRICE_BASE_QUERY: &str = r#"
            SELECT
                market_id,
                toFloat64(yes_price) as yes_price, 
                toFloat64(no_price) as no_price, 
                ts,
                created_at
            FROM market_price_data WHERE market_id = ?
        "#;

    pub const MARKET_VOLUME_BASE_QUERY: &str = r#"
            SELECT
                market_id,

                -- YES - BUY
                toFloat64(SUM(if(outcome = 'yes' AND side = 'buy', quantity, 0))) AS yes_buy_qty,
                toFloat64(SUM(if(outcome = 'yes' AND side = 'buy', amount, 0))) AS yes_buy_usd,

                -- YES - SELL
                toFloat64(SUM(if(outcome = 'yes' AND side = 'sell', quantity, 0))) AS yes_sell_qty,
                toFloat64(SUM(if(outcome = 'yes' AND side = 'sell', amount, 0))) AS yes_sell_usd,

                -- NO - BUY
                toFloat64(SUM(if(outcome = 'no' AND side = 'buy', quantity, 0))) AS no_buy_qty,
                toFloat64(SUM(if(outcome = 'no' AND side = 'buy', amount, 0))) AS no_buy_usd,

                -- NO - SELL
                toFloat64(SUM(if(outcome = 'no' AND side = 'sell', quantity, 0))) AS no_sell_qty,
                toFloat64(SUM(if(outcome = 'no' AND side = 'sell', amount, 0))) AS no_sell_usd

            FROM market_volume_data
            WHERE
                market_id = ? AND
                ts >= now() - INTERVAL ?
            GROUP BY market_id
        "#;

    pub const MARKET_LATEST_PRICE_QUERY: &str = r#"
            SELECT
                market_id,
                toFloat64(argMax(yes_price, ts)) AS latest_yes_price,
                toFloat64(argMax(no_price, ts)) AS latest_no_price
            FROM market_price_data
            WHERE market_id = ?
            GROUP BY market_id
        "#;
}
