use crate::spot::{
    v3::{ApiResponse, ApiResult},
    MexcSpotApiTrait,
};
use async_trait::async_trait;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultsSymbolsOutput {
    pub code: i32,
    pub data: Vec<String>,
    pub msg: Option<String>,
}

#[async_trait]
pub trait DefaultSymbolsEndpoint {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> DefaultSymbolsEndpoint for T {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput> {
        let endpoint = format!("{}/api/v3/defaultSymbols", self.endpoint().as_ref());
        let response = self.reqwest_client().get(&endpoint).send().await?;
        let api_response = response
            .json::<ApiResponse<DefaultsSymbolsOutput>>()
            .await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_default_symbols() {
        let client = MexcSpotApiClient::default();
        let result = client.time().await;
        assert!(result.is_ok());
    }
}
