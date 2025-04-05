use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvgParams<'a> {
    /// Symbol
    pub symbol: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvgOutput {
    pub mins: u64,
    pub price: Decimal,
}

#[async_trait]
pub trait AvgEndpoint {
    /// Order book
    async fn avg_price(&self, params: AvgParams<'_>) -> ApiResult<AvgOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> AvgEndpoint for T {
    async fn avg_price(&self, params: AvgParams<'_>) -> ApiResult<AvgOutput> {
        let endpoint = format!("{}/api/v3/avgPrice", self.endpoint().as_ref());
        let response = self.reqwest_client().get(&endpoint).query(&params).send().await?;
        let api_response = response.json::<ApiResponse<AvgOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_depth() {
        let client = MexcSpotApiClient::default();
        let avg_params = AvgParams {
            symbol: "BTCUSDT"
        };
        let result = client.avg_price(avg_params).await;
        assert!(result.is_ok());
    }
}
