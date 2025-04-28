use crate::futures::auth::SignRequestParamsKind;
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::AccountAsset;
use crate::futures::MexcFuturesApiClientWithAuthentication;
use async_trait::async_trait;

#[async_trait]
pub trait GetAccountAsset {
    async fn get_account_asset<'a>(&self, currency: &'a str) -> ApiResult<AccountAsset>;
}

#[async_trait]
impl GetAccountAsset for MexcFuturesApiClientWithAuthentication {
    async fn get_account_asset<'a>(&self, currency: &'a str) -> ApiResult<AccountAsset> {
        let url = format!(
            "{}/api/v1/private/account/asset/{}",
            self.endpoint.as_ref(),
            currency
        );
        let auth_header_map = self.get_auth_header_map(&(), SignRequestParamsKind::Query)?;
        let response = self
            .reqwest_client
            .get(&url)
            .headers(auth_header_map)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<AccountAsset>>().await?;
        api_response.into_api_result()
    }
}
