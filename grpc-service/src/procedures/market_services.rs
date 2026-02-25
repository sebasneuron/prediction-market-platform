use std::str::FromStr;

use db_service::schema::{
    enums::MarketStatus, market::Market as SchemaMarket, user_holdings::UserHoldings,
    user_trades::UserTrades,
};
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};
use utility_helpers::redis::keys::RedisKey;

use crate::{
    generated::markets::{
        GetMarketBookResponse, GetMarketByIdResponse, GetMarketDataRequest,
        GetMarketTradesResponse, GetPaginatedMarketResponse, GetTopHoldersResponse,
        RequestForMarketBook, RequestWithMarketId, RequestWithMarketIdAndPageRequest,
        market_service_server::MarketService,
    },
    procedures::{from_db_market, to_resp_for_market_book},
    state::SafeState,
    utils::{
        clickhouse_queries::{
            MARKET_LATEST_PRICE_QUERY, MARKET_VOLUME_BASE_QUERY, ORDER_BOOK_INITIALS,
        },
        clickhouse_schema::{GetOrderBook, MarketPriceResponse, VolumeData},
    },
    validate_numbers, validate_strings,
};

pub struct MarketServiceStub {
    pub state: SafeState,
}

#[tonic::async_trait]
impl MarketService for MarketServiceStub {
    async fn get_market_data(
        &self,
        req: Request<GetMarketDataRequest>,
    ) -> Result<Response<GetPaginatedMarketResponse>, Status> {
        let page_info = req.get_ref().page_request.clone();
        if page_info.is_none() {
            return Err(Status::invalid_argument("Page request cannot be empty"));
        }
        let page_info = page_info.unwrap();
        let page_no = page_info.page;
        let page_size = page_info.page_size;

        let market_status: MarketStatus = match req.get_ref().market_status {
            0 => MarketStatus::SETTLED,
            1 => MarketStatus::OPEN,
            2 => MarketStatus::CLOSED,
            3 => MarketStatus::SETTLED,
            _ => return Err(Status::invalid_argument("Invalid market status")),
        };

        validate_numbers!(page_no);
        validate_numbers!(page_size);

        let key = RedisKey::Markets(page_no, page_size, market_status as u64);
        if page_no == 0 || page_size == 0 {
            return Err(Status::invalid_argument(
                "Page number and size must be greater than 0",
            ));
        }

        let markets = self
            .state
            .redis_helper
            .get_or_set_cache(
                key,
                || async {
                    Ok(SchemaMarket::get_all_market_by_status_paginated(
                        &self.state.db_pool,
                        market_status,
                        page_no,
                        page_size,
                    )
                    .await?)
                },
                Some(60), // Cache for 60 seconds
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get market {e}")))?;

        let response = GetPaginatedMarketResponse {
            markets: markets
                .items
                .iter()
                .map(|item| from_db_market(item, 0.5, 0.5))
                .collect(),
            page_info: Some(markets.page_info.into()),
        };

        Ok(Response::new(response))
    }

    async fn get_market_by_id(
        &self,
        req: Request<RequestWithMarketId>,
    ) -> Result<Response<GetMarketByIdResponse>, Status> {
        let market_id = req.into_inner().market_id;
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let key = RedisKey::Market(market_id);

        let market = self
            .state
            .redis_helper
            .get_or_set_cache(
                key,
                || async {
                    Ok(SchemaMarket::get_market_by_id(&self.state.db_pool, &market_id).await?)
                },
                Some(60), // Cache for 60 seconds
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get market id {e}")))?;

        if let Some(market) = market {
            let market = from_db_market(&market, 0.5, 0.5);

            // we are not caching the volume info in Redis, as it changes frequently

            let get_volume_info_future = self
                .state
                .clickhouse_client
                .query(MARKET_VOLUME_BASE_QUERY)
                .bind(market_id)
                .bind("1 DAY") // Assuming we want the last 24 hours of volume data
                .fetch_optional::<VolumeData>();
            let get_market_price_future = self
                .state
                .clickhouse_client
                .query(MARKET_LATEST_PRICE_QUERY)
                .bind(market_id)
                .fetch_optional::<MarketPriceResponse>();

            let (volume_info, market_price) =
                tokio::try_join!(get_volume_info_future, get_market_price_future)
                    .map_err(|e| Status::internal(format!("Failed to fetch market data: {}", e)))?;

            let (volume_info_resp, market_price_resp) =
                if let (Some(volume_info), Some(market_price)) = (volume_info, market_price) {
                    (volume_info, market_price)
                } else {
                    (VolumeData::default(), MarketPriceResponse::default())
                };

            let response = GetMarketByIdResponse {
                market: Some(market),
                volume_info: Some(volume_info_resp.into()),
                market_price: Some(market_price_resp.into()),
            };
            return Ok(Response::new(response));
        }

        Err(Status::not_found(format!(
            "Market with {market_id} not found"
        )))
    }

    async fn get_market_book(
        &self,
        req: Request<RequestForMarketBook>,
    ) -> Result<Response<GetMarketBookResponse>, Status> {
        let market_id = &req.get_ref().market_id;
        let depth = req.get_ref().depth;
        validate_numbers!(depth);
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let order_book_initials = self
            .state
            .clickhouse_client
            .query(ORDER_BOOK_INITIALS)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(market_id)
            .fetch_optional::<GetOrderBook>()
            .await
            .map_err(|e| Status::internal(format!("Failed to fetch market book: {}", e)))?;

        if order_book_initials.is_none() {
            return Err(Status::not_found(format!(
                "Market book for market id {market_id} not found"
            )));
        }

        let order_book = to_resp_for_market_book(order_book_initials.unwrap());
        let response = Response::new(order_book);

        Ok(response)
    }

    async fn get_top_holders(
        &self,
        req: Request<RequestWithMarketId>,
    ) -> Result<Response<GetTopHoldersResponse>, Status> {
        let market_id = req.into_inner().market_id;
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let top_holders = UserHoldings::get_top_holders(
            &self.state.db_pool,
            market_id,
            self.state.admin_username.clone(),
            10,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get top holders: {}", e)))?;

        let response = GetTopHoldersResponse {
            market_id: market_id.to_string(),
            top_holders: top_holders.into_iter().map(Into::into).collect(),
        };

        Ok(Response::new(response))
    }

    async fn get_market_trades(
        &self,
        req: Request<RequestWithMarketIdAndPageRequest>,
    ) -> Result<Response<GetMarketTradesResponse>, Status> {
        let market_id = req.get_ref().market_id.clone();
        let page_request = req.get_ref().page_request;

        if page_request.is_none() {
            return Err(Status::invalid_argument("Page request cannot be empty"));
        }
        let page_request = page_request.unwrap();

        validate_strings!(market_id);
        validate_numbers!(page_request.page);
        validate_numbers!(page_request.page_size);
        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let paginated_response = UserTrades::get_market_trades_paginated(
            market_id,
            self.state.admin_username.clone(),
            page_request.page,
            page_request.page_size,
            &self.state.db_pool,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get market trades: {}", e)))?;

        let response = GetMarketTradesResponse {
            market_id: market_id.to_string(),
            trades: paginated_response
                .items
                .into_iter()
                .map(Into::into)
                .collect(),
            page_info: Some(paginated_response.page_info.into()),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::types::Uuid;

    use crate::utils::clickhouse_schema::{GetOrderBook, MarketPriceResponse, VolumeData};

    #[tokio::test]
    #[ignore = "Requires market id"]
    async fn test_get_market_data() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");
        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();
        let depth = 10;

        let resp = client
            .query(
                r#"
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
            "#,
            )
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(market_id)
            .fetch_optional::<GetOrderBook>()
            .await
            .inspect_err(|e| {
                println!("Error fetching market data: {}", e);
            })
            .unwrap();

        assert!(resp.is_some(), "Response should not be empty");
    }

    #[tokio::test]
    #[ignore = "Requires market id"]
    async fn test_get_market_volume() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");

        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();

        let result = client
            .query(
                r#"
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
                "#,
            )
            .bind(market_id)
            .bind("1 DAY") 
            .fetch_one::<VolumeData>()
            .await;

        let result = result
            .inspect_err(|e| {
                println!("Error fetching market volume data: {}", e);
            })
            .unwrap();
        println!("Market Volume Data: {:#?}", result);
        assert_eq!(result.market_id, market_id);
    }

    #[tokio::test]
    async fn test_get_latest_market_price() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");

        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();

        let result = client
            .query(
                r#"
                SELECT
                    market_id,
                    toFloat64(argMax(yes_price, ts)) AS latest_yes_price,
                    toFloat64(argMax(no_price, ts)) AS latest_no_price
                FROM market_price_data
                WHERE market_id = ?
                GROUP BY market_id
                "#,
            )
            .bind(market_id)
            .fetch_optional::<MarketPriceResponse>()
            .await;

        let result = result
            .inspect_err(|e| {
                println!("Error fetching latest market price: {}", e);
            })
            .unwrap();
        println!("Latest Market Price: {:#?}", result);
    }
}
