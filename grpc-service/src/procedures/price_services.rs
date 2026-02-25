use sqlx::types::Uuid;
use tonic::{Request, Response, Status};

use crate::{
    generated::{
        common::Timeframe,
        price::{
            GetMarketPriceDataWithinIntervalResponse, GetPriceDataWithinIntervalRequest, PriceData,
            price_service_server::PriceService,
        },
    },
    state::SafeState,
    utils::{clickhouse_queries::MARKET_PRICE_BASE_QUERY, clickhouse_schema::GetMarketPrices},
    validate_numbers, validate_strings,
};

pub struct PriceServiceStub {
    pub state: SafeState,
}

#[tonic::async_trait]
impl PriceService for PriceServiceStub {
    async fn get_price_data_within_interval(
        &self,
        request: Request<GetPriceDataWithinIntervalRequest>,
    ) -> Result<Response<GetMarketPriceDataWithinIntervalResponse>, Status> {
        let req = request.into_inner();
        let market_id = req.market_id;
        let timeframe = req.timeframe;

        validate_strings!(market_id);
        validate_numbers!(timeframe);

        let client = &self.state.clickhouse_client;
        let time_range = Timeframe::try_from(timeframe).map_err(|_| {
            Status::invalid_argument("Invalid timeframe provided. Must be a valid Timeframe enum.")
        })?;

        let base_query = MARKET_PRICE_BASE_QUERY;

        let query = match time_range.get_start_time() {
            Some(start_time) => format!(
                "{} AND ts >= '{}' ORDER BY ts ASC",
                base_query,
                start_time.format("%Y-%m-%d %H:%M:%S")
            ),
            None => format!("{} ORDER BY ts ASC", base_query),
        };

        let resp = client
            .query(&query)
            .bind(market_id)
            .fetch_all::<GetMarketPrices>()
            .await
            .map_err(|e| Status::internal(format!("Database query failed: {}", e)))?;

        let market_id = if let Some(id) = resp.first().map(|r| r.market_id) {
            id.to_string()
        } else {
            Uuid::nil().to_string()
        };

        let response = GetMarketPriceDataWithinIntervalResponse {
            market_id,
            price_data: resp
                .into_iter()
                .map(|data| PriceData {
                    yes_price: data.yes_price,
                    no_price: data.no_price,
                    timestamp: data.ts.timestamp_millis() as u64,
                })
                .collect(),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod test {

    use crate::procedures::price_services::GetMarketPrices;

    #[tokio::test]
    #[ignore = "Skip"]
    async fn test_clickhouse_data() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");

        let another_query = client
            .query(
                r#"
                 SELECT 
                    market_id, 
                    toFloat64(yes_price) AS yes_price,
                    toFloat64(no_price) AS no_price,
                    ts,
                    created_at
                FROM market_price_data
                "#,
            )
            .fetch_all::<GetMarketPrices>()
            .await;
        println!("Another query response: {another_query:#?}");

        assert!(another_query.is_ok(), "Query should succeed");
    }
}
