use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepthParams<'a> {
    /// Symbol
    pub symbol: &'a str,
    /// Return number default 100; max 5000
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PriceAndQuantity {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepthOutput {
    pub last_update_id: u64,
    pub bids: Vec<PriceAndQuantity>,
    pub asks: Vec<PriceAndQuantity>,
}

#[async_trait]
pub trait DepthEndpoint {
    /// Order book
    async fn depth(&self, params: DepthParams<'_>) -> ApiResult<DepthOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> DepthEndpoint for T {
    async fn depth(&self, params: DepthParams<'_>) -> ApiResult<DepthOutput> {
        let endpoint = format!("{}/api/v3/depth", self.endpoint().as_ref());
        let response = self
            .reqwest_client()
            .get(&endpoint)
            .query(&params)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<DepthOutput>>().await?;
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
        let depth_params = DepthParams {
            symbol: "BTCUSDT",
            limit: None,
        };
        let result = client.depth(depth_params).await;
        assert!(result.is_ok());
    }
}
