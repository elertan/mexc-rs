use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::{MexcApiClient, MexcApiClientWithAuthentication, MexcApiEndpoint};
use crate::v3::ApiV3Result;
use crate::v3::enums::TradeType;

#[derive(Debug, serde::Serialize)]
pub struct TradesParams<'a> {
    pub symbol: &'a str,
    /// Default 500; max 1000.
    pub limit: Option<u32>,
}

#[derive(Debug)]
pub struct TradesOutput {
    pub trades: Vec<Trade>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Currently always filled with null
    pub id: Option<serde_json::Value>,
    pub price: BigDecimal,
    #[serde(rename = "qty")]
    pub quantity: BigDecimal,
    #[serde(rename = "quoteQty")]
    pub quote_quantity: BigDecimal,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
    pub trade_type: TradeType,
}

#[async_trait]
pub trait TradesEndpoint {
    async fn trades(&self, params: TradesParams<'_>) -> ApiV3Result<TradesOutput>;
}

async fn trades_impl(
    endpoint: &MexcApiEndpoint,
    client: &reqwest::Client,
    params: TradesParams<'_>,
) -> ApiV3Result<TradesOutput> {
    let endpoint = format!("{}/api/v3/trades", endpoint.as_ref());
    let response = client.get(&endpoint).query(&params).send().await?;
    let trades = response.json::<Vec<Trade>>().await?;

    Ok(TradesOutput {
        trades
    })
}

#[async_trait]
impl TradesEndpoint for MexcApiClient {
    async fn trades(&self, params: TradesParams<'_>) -> ApiV3Result<TradesOutput> {
        trades_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}

#[async_trait]
impl TradesEndpoint for MexcApiClientWithAuthentication {
    async fn trades(&self, params: TradesParams<'_>) -> ApiV3Result<TradesOutput> {
        trades_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trades() {
        let client = MexcApiClient::default();
        let params = TradesParams {
            symbol: "KASUSDT",
            limit: Some(1000),
        };
        let result = client.trades(params).await;
        assert!(result.is_ok());
    }
}
