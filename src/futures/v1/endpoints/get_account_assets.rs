use async_trait::async_trait;
use bigdecimal::BigDecimal;
use crate::futures::{MexcFuturesApiClientWithAuthentication};
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    pub currency: String,
    pub position_margin: BigDecimal,
    pub frozen_balance: BigDecimal,
    pub available_balance: BigDecimal,
    pub cash_balance: BigDecimal,
    pub equity: BigDecimal,
    pub unrealized: BigDecimal,
}

#[async_trait]
pub trait GetAccountAssets {
    async fn get_account_assets(&self) -> ApiResult<Vec<AccountAsset>>;
}


#[async_trait]
impl GetAccountAssets for MexcFuturesApiClientWithAuthentication {
    async fn get_account_assets(&self) -> ApiResult<Vec<AccountAsset>> {
        let url = format!("{}/api/v1/private/account/assets", self.endpoint.as_ref());
        let auth_header_map = self.get_auth_header_map(&())?;
        let response = self.reqwest_client.get(&url).headers(auth_header_map).send().await?;
        let api_response = response.json::<ApiResponse<Vec<AccountAsset>>>().await?;
        api_response.into_api_result()
    }
}
