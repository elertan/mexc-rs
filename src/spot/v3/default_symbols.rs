use async_trait::async_trait;
use crate::spot::{MexcSpotApiClient, MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use crate::spot::v3::{ApiResponse, ApiResult};


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

async fn default_symbols_impl(
    endpoint: &MexcSpotApiEndpoint,
    client: &reqwest::Client,
) -> ApiResult<DefaultsSymbolsOutput> {
    let endpoint = format!("{}/api/v3/defaultSymbols", endpoint.as_ref());
    let response = client.get(&endpoint).send().await?;
    let response = response.json::<ApiResponse<DefaultsSymbolsOutput>>().await?;
    let output = response.into_api_result()?;

    Ok(output)
}

#[async_trait]
impl DefaultSymbolsEndpoint for MexcSpotApiClient {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput> {
        default_symbols_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl DefaultSymbolsEndpoint for MexcSpotApiClientWithAuthentication {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput> {
        default_symbols_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_symbols() {
        let client = MexcSpotApiClient::default();
        let result = client.time().await;
        assert!(result.is_ok());
    }
}
