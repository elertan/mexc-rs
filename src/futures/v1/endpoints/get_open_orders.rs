use async_trait::async_trait;
use crate::futures::{MexcFuturesApiClientWithAuthentication};
use crate::futures::auth::SignRequestParamsKind;
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::{OpenOrder};

#[derive(Debug)]
pub struct GetOpenOrdersParams {
    pub page_num: u32,
    pub page_size: u32,
}

#[async_trait]
pub trait GetOpenOrders {
    async fn get_open_orders<'a>(&self, params: GetOpenOrdersParams) -> ApiResult<Vec<OpenOrder>>;
}

#[derive(Debug, serde::Serialize)]
pub struct QueryParams {
    pub page_num: u32,
    pub page_size: u32,
}

#[async_trait]
impl GetOpenOrders for MexcFuturesApiClientWithAuthentication {
    async fn get_open_orders<'a>(&self, params: GetOpenOrdersParams) -> ApiResult<Vec<OpenOrder>> {
        let url = format!("{}/api/v1/private/order/list/open_orders", self.endpoint.as_ref());
        let query = QueryParams { page_num: params.page_num, page_size: params.page_size };
        let auth_header_map = self.get_auth_header_map(&query, SignRequestParamsKind::Query)?;
        let response = self.reqwest_client.get(&url).query(&query).headers(auth_header_map).send().await?;
        let api_response = response.json::<ApiResponse<Vec<OpenOrder>>>().await?;
        api_response.into_api_result()
    }
}
