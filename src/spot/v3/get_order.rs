use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::spot::MexcSpotApiClientWithAuthentication;
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::v3::models::Order;

#[derive(Debug)]
pub struct GetOrderParams<'a> {
    pub symbol: &'a str,
    pub order_id: Option<&'a str>,
    pub original_client_order_id: Option<&'a str>,
    pub new_client_order_id: Option<&'a str>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderQuery<'a> {
    pub symbol: &'a str,
    pub order_id: Option<&'a str>,
    #[serde(rename = "origClientOrderId")]
    pub original_client_order_id: Option<&'a str>,
    pub new_client_order_id: Option<&'a str>,
    /// Max 60000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<GetOrderParams<'a>> for GetOrderQuery<'a> {
    fn from(params: GetOrderParams<'a>) -> Self {
        Self {
            symbol: params.symbol,
            order_id: params.order_id,
            original_client_order_id: params.original_client_order_id,
            new_client_order_id: params.new_client_order_id,
            recv_window: None,
            timestamp: Utc::now(),
        }
    }
}


#[async_trait]
pub trait GetOrderEndpoint {
    async fn get_order(&self, params: GetOrderParams<'_>) -> ApiResult<Order>;
}

#[async_trait]
impl GetOrderEndpoint for MexcSpotApiClientWithAuthentication {
    async fn get_order(&self, params: GetOrderParams<'_>) -> ApiResult<Order> {
        let endpoint = format!("{}/api/v3/order", self.endpoint.as_ref());
        let query = GetOrderQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self.reqwest_client.get(&endpoint).query(&query_with_signature).send().await?;
        let api_response = response.json::<ApiResponse<Order>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_order() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let params = GetOrderParams {
            symbol: "KASUSDT",
            order_id: None,
            original_client_order_id: Some("MY_ORDER_ID"),
            new_client_order_id: None,
        };
        let result = client.get_order(params).await;
        assert!(result.is_ok());
    }
}
