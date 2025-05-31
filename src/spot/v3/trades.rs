use crate::spot::v3::enums::TradeType;
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize)]
pub struct TradesParams<'a> {
    pub symbol: &'a str,
    /// Default 500; max 1000.
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TradesOutput {
    pub trades: Vec<Trade>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Currently always filled with null
    pub id: Option<serde_json::Value>,
    pub price: Decimal,
    #[serde(rename = "qty")]
    pub quantity: Decimal,
    #[serde(rename = "quoteQty")]
    pub quote_quantity: Decimal,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
    pub trade_type: TradeType,
}

#[async_trait]
pub trait TradesEndpoint {
    async fn trades(&self, params: TradesParams<'_>) -> ApiResult<TradesOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> TradesEndpoint for T {
    async fn trades(&self, params: TradesParams<'_>) -> ApiResult<TradesOutput> {
        let endpoint = format!("{}/api/v3/trades", self.endpoint().as_ref());
        let response = self
            .reqwest_client()
            .get(&endpoint)
            .query(&params)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<Vec<Trade>>>().await?;
        let trades = api_response.into_api_result()?;

        Ok(TradesOutput { trades })
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_trades() {
        let client = MexcSpotApiClient::default();
        let params = TradesParams {
            symbol: "KASUSDT",
            limit: Some(1000),
        };
        let result = client.trades(params).await;
        assert!(result.is_ok());
    }
}
