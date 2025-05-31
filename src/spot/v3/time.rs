use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeOutput {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub server_time: DateTime<Utc>,
}

#[async_trait]
pub trait TimeEndpoint {
    async fn time(&self) -> ApiResult<TimeOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> TimeEndpoint for T {
    async fn time(&self) -> ApiResult<TimeOutput> {
        let endpoint = format!("{}/api/v3/time", self.endpoint().as_ref());
        let response = self.reqwest_client().get(&endpoint).send().await?;
        let api_response = response.json::<ApiResponse<TimeOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_time() {
        let client = MexcSpotApiClient::default();
        let result = client.time().await;
        assert!(result.is_ok());
    }
}
