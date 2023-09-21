use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use crate::futures::{MexcFuturesApiClient, MexcFuturesApiClientWithAuthentication, MexcFuturesApiEndpoint};
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;

#[async_trait]
pub trait GetAccountAssets {
    async fn get_account_assets(&self) -> ApiResult<DateTime<Utc>>;
}


#[async_trait]
impl GetAccountAssets for MexcFuturesApiClientWithAuthentication {
    async fn get_account_assets(&self) -> ApiResult<DateTime<Utc>> {
        let url = format!("{}/api/v1/private/account/assets", self.endpoint.as_ref());
        let auth_header_map = self.get_auth_header_map(&())?;
        let response = self.reqwest_client.get(&url).headers(auth_header_map).send().await?;
        let json = response.text().await?;
        tracing::debug!("{}", json);

        // let api_response = response.json::<ApiResponse<i64>>().await?;
        // let timestamp = api_response.into_api_result()?;


        todo!();
        // Ok(Utc.timestamp_millis_opt(timestamp).unwrap())
    }
}
