use crate::futures::auth::SignRequestParamsKind;
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::OpenPosition;
use crate::futures::MexcFuturesApiClientWithAuthentication;
use async_trait::async_trait;

#[async_trait]
pub trait GetOpenPositions {
    async fn get_open_positions<'a>(&self, symbol: Option<&'a str>)
        -> ApiResult<Vec<OpenPosition>>;
}

#[derive(Debug, serde::Serialize)]
pub struct QueryParams<'a> {
    pub symbol: Option<&'a str>,
}

#[async_trait]
impl GetOpenPositions for MexcFuturesApiClientWithAuthentication {
    async fn get_open_positions<'a>(
        &self,
        symbol: Option<&'a str>,
    ) -> ApiResult<Vec<OpenPosition>> {
        let url = format!(
            "{}/api/v1/private/position/open_positions",
            self.endpoint.as_ref()
        );
        let query = QueryParams { symbol };
        let auth_header_map = self.get_auth_header_map(&query, SignRequestParamsKind::Query)?;
        let response = self
            .reqwest_client
            .get(&url)
            .query(&query)
            .headers(auth_header_map)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<Vec<OpenPosition>>>().await?;
        api_response.into_api_result()
    }
}
