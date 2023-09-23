use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::spot::{MexcSpotApiClientWithAuthentication};
use crate::spot::v3::{ApiResponse, ApiResult};

#[derive(Debug)]
pub struct KeepAliveUserDataStreamParams<'a> {
    pub listen_key: &'a str,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAliveUserDataStreamQuery<'a> {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    timestamp: DateTime<Utc>,
    listen_key: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAliveUserDataStreamOutput {
    pub listen_key: String,
}

#[async_trait]
pub trait KeepAliveUserDataStreamEndpoint {
    async fn keep_alive_user_data_stream(&self, params: KeepAliveUserDataStreamParams<'_>) -> ApiResult<KeepAliveUserDataStreamOutput>;
}

#[async_trait]
impl KeepAliveUserDataStreamEndpoint for MexcSpotApiClientWithAuthentication {
    async fn keep_alive_user_data_stream(&self, params: KeepAliveUserDataStreamParams<'_>) -> ApiResult<KeepAliveUserDataStreamOutput> {
        let url = format!("{}/api/v3/userDataStream", self.endpoint.as_ref());
        let query = KeepAliveUserDataStreamQuery {
            timestamp: Utc::now(),
            listen_key: params.listen_key,
        };
        let query = self.sign_query(&query)?;
        let response = self.reqwest_client.put(&url).query(&query).send().await?;
        let api_response = response.json::<ApiResponse<KeepAliveUserDataStreamOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}
