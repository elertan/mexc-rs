use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::{MexcApiClient, MexcApiClientWithAuthentication, MexcApiEndpoint};
use crate::v3::ApiV3Result;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOutput {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub server_time: DateTime<Utc>,
}

#[async_trait]
pub trait TimeEndpoint {
    async fn time(&self) -> ApiV3Result<TimeOutput>;
}

async fn time_impl(
    endpoint: &MexcApiEndpoint,
    client: &reqwest::Client,
) -> ApiV3Result<TimeOutput> {
    let endpoint = format!("{}/api/v3/time", endpoint.as_ref());
    let response = client.get(&endpoint).send().await?;
    let output = response.json::<TimeOutput>().await?;

    Ok(output)
}

#[async_trait]
impl TimeEndpoint for MexcApiClient {
    async fn time(&self) -> ApiV3Result<TimeOutput> {
        time_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl TimeEndpoint for MexcApiClientWithAuthentication {
    async fn time(&self) -> ApiV3Result<TimeOutput> {
        time_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_time() {
        let client = MexcApiClient::default();
        let result = client.time().await;
        assert!(result.is_ok());
    }
}
