use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::{MexcApiClientWithAuthentication};
use crate::v3::{ApiResponse, ApiResult};
use crate::v3::enums::{OrderSide, OrderStatus, OrderType};

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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderOutput {
    pub symbol: String,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub price: BigDecimal,
    #[serde(rename = "origQty")]
    pub original_quantity: BigDecimal,
    #[serde(rename = "executedQty")]
    pub executed_quantity: BigDecimal,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cummulative_quote_quantity: BigDecimal,
    pub status: OrderStatus,
    pub time_in_force: Option<String>,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub stop_price: BigDecimal,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub update_time: DateTime<Utc>,
    pub is_working: bool,
    #[serde(rename = "origQuoteOrderQty")]
    pub original_quote_order_qty: BigDecimal,
}


#[async_trait]
pub trait GetOrderEndpoint {
    async fn get_order(&self, params: GetOrderParams<'_>) -> ApiResult<GetOrderOutput>;
}

#[async_trait]
impl GetOrderEndpoint for MexcApiClientWithAuthentication {
    async fn get_order(&self, params: GetOrderParams<'_>) -> ApiResult<GetOrderOutput> {
        let endpoint = format!("{}/api/v3/order", self.endpoint.as_ref());
        let query = GetOrderQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self.reqwest_client.get(&endpoint).query(&query_with_signature).send().await?;
        let api_response = response.json::<ApiResponse<GetOrderOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_order() {
        let client = MexcApiClientWithAuthentication::new_for_test();
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
