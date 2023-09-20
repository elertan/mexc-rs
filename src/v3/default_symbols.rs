use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::{MexcApiClient, MexcApiClientWithAuthentication, MexcApiEndpoint};
use crate::v3::ApiResult;

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
    endpoint: &MexcApiEndpoint,
    client: &reqwest::Client,
) -> ApiResult<DefaultsSymbolsOutput> {
    let endpoint = format!("{}/api/v3/defaultSymbols", endpoint.as_ref());
    let response = client.get(&endpoint).send().await?;
    let output = response.json::<DefaultsSymbolsOutput>().await?;

    Ok(output)
}

#[async_trait]
impl DefaultSymbolsEndpoint for MexcApiClient {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput> {
        default_symbols_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl DefaultSymbolsEndpoint for MexcApiClientWithAuthentication {
    async fn time(&self) -> ApiResult<DefaultsSymbolsOutput> {
        default_symbols_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_symbols() {
        let client = MexcApiClient::default();
        let result = client.time().await;
        assert!(result.is_ok());
    }
}
