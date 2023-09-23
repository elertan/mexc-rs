use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::spot::{MexcSpotApiClientWithAuthentication, QueryWithSignature};
use crate::spot::v3::{ApiResponse, ApiResult};

#[derive(Debug, serde::Serialize)]
pub struct CreateUserDataStreamQuery {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    timestamp: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDataStreamOutput {
    pub listen_key: String,
}

#[async_trait]
pub trait CreateUserDataStreamEndpoint {
    async fn create_user_data_stream(&self) -> ApiResult<CreateUserDataStreamOutput>;
}

#[async_trait]
impl CreateUserDataStreamEndpoint for MexcSpotApiClientWithAuthentication {
    async fn create_user_data_stream(&self) -> ApiResult<CreateUserDataStreamOutput> {
        let url = format!("{}/api/v3/userDataStream", self.endpoint.as_ref());
        let query = CreateUserDataStreamQuery {
            timestamp: Utc::now(),
        };
        let query = self.sign_query(&query)?;
        let response = self.reqwest_client.post(&url).query(&query).send().await?;
        let api_response = response.json::<ApiResponse<CreateUserDataStreamOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}
