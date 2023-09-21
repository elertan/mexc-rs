use async_trait::async_trait;
use crate::futures::{MexcFuturesApiClientWithAuthentication};
use crate::futures::auth::SignRequestParamsKind;
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::AccountAsset;

#[async_trait]
pub trait GetAccountAssets {
    async fn get_account_assets(&self) -> ApiResult<Vec<AccountAsset>>;
}

#[async_trait]
impl GetAccountAssets for MexcFuturesApiClientWithAuthentication {
    async fn get_account_assets(&self) -> ApiResult<Vec<AccountAsset>> {
        let url = format!("{}/api/v1/private/account/assets", self.endpoint.as_ref());
        let auth_header_map = self.get_auth_header_map(&(), SignRequestParamsKind::Query)?;
        let response = self.reqwest_client.get(&url).headers(auth_header_map).send().await?;
        let api_response = response.json::<ApiResponse<Vec<AccountAsset>>>().await?;
        api_response.into_api_result()
    }
}
