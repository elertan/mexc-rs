use crate::spot::v3::enums::{OrderSide, OrderStatus, OrderType};
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct QueryOrderParams<'a> {
    pub symbol: &'a str,
    pub order_id: Option<&'a str>,
    pub original_client_order_id: Option<&'a str>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderQuery<'a> {
    pub symbol: &'a str,
    pub order_id: Option<&'a str>,
    #[serde(rename = "origClientOrderId")]
    pub original_client_order_id: Option<&'a str>,
    /// Max 60000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<QueryOrderParams<'a>> for QueryOrderQuery<'a> {
    fn from(params: QueryOrderParams<'a>) -> Self {
        Self {
            symbol: params.symbol,
            order_id: params.order_id,
            original_client_order_id: params.original_client_order_id,
            recv_window: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderOutput {
    pub symbol: String,
    #[serde(rename = "origClientOrderId")]
    pub original_client_order_id: Option<String>,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: Option<String>,
    pub price: Decimal,
    #[serde(rename = "origQty")]
    pub original_quantity: Decimal,
    #[serde(rename = "executedQty")]
    pub executed_quantity: Decimal,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cummulative_quote_quantity: Decimal,
    pub status: OrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    #[serde(rename = "stopPrice")]
    pub stop_price: Decimal,
    pub time: DateTime<Utc>,
    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>,
    #[serde(rename = "isWorking")]
    pub is_working: bool,
}

#[async_trait]
pub trait QueryOrderEndpoint {
    async fn query_order(&self, params: QueryOrderParams<'_>) -> ApiResult<QueryOrderOutput>;
}

#[async_trait]
impl QueryOrderEndpoint for MexcSpotApiClientWithAuthentication {
    async fn query_order(&self, params: QueryOrderParams<'_>) -> ApiResult<QueryOrderOutput> {
        let endpoint = format!("{}/api/v3/order", self.endpoint.as_ref());
        let query = QueryOrderQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self
            .reqwest_client
            .delete(&endpoint)
            .query(&query_with_signature)
            .send()
            .await?;

        let api_response = response.json::<ApiResponse<QueryOrderOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn query_order() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let params = QueryOrderParams {
            symbol: "KASUSDT",
            order_id: None,
            original_client_order_id: Some("MY_ORDER_ID"),
        };
        let result = client.query_order(params).await;
        assert!(result.is_ok());
    }
}
