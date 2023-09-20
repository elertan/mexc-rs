use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use crate::futures::{MexcFuturesApiClient, MexcFuturesApiClientWithAuthentication, MexcFuturesApiEndpoint};
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;

#[async_trait]
pub trait GetServerTime {
    async fn get_server_time(&self) -> ApiResult<DateTime<Utc>>;
}

async fn default_impl(endpoint: &MexcFuturesApiEndpoint, reqwest: &Client) -> ApiResult<DateTime<Utc>> {
    let url = format!("{}/api/v1/contract/ping", endpoint.as_ref());
    let response = reqwest.get(&url).send().await?;
    let api_response = response.json::<ApiResponse<i64>>().await?;
    let timestamp = api_response.into_api_result()?;

    Ok(Utc.timestamp_millis_opt(timestamp).unwrap())
}

#[async_trait]
impl GetServerTime for MexcFuturesApiClient {
    async fn get_server_time(&self) -> ApiResult<DateTime<Utc>> {
        default_impl(&self.endpoint, &self.reqwest_client).await
    }
}

#[async_trait]
impl GetServerTime for MexcFuturesApiClientWithAuthentication {
    async fn get_server_time(&self) -> ApiResult<DateTime<Utc>> {
        default_impl(&self.endpoint, &self.reqwest_client).await
    }
}
