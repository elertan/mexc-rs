use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::spot::MexcSpotApiClientWithAuthentication;
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::v3::enums::{OrderSide, OrderStatus, OrderType};

#[derive(Debug)]
pub struct CancelOrderParams<'a> {
    pub symbol: &'a str,
    pub order_id: Option<&'a str>,
    pub original_client_order_id: Option<&'a str>,
    pub new_client_order_id: Option<&'a str>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderQuery<'a> {
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

impl<'a> From<CancelOrderParams<'a>> for CancelOrderQuery<'a> {
    fn from(params: CancelOrderParams<'a>) -> Self {
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
pub struct CancelOrderOutput {
    pub symbol: String,
    #[serde(rename = "origClientOrderId")]
    pub original_client_order_id: Option<String>,
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
}


#[async_trait]
pub trait CancelOrderEndpoint {
    async fn cancel_order(&self, params: CancelOrderParams<'_>) -> ApiResult<CancelOrderOutput>;
}

#[async_trait]
impl CancelOrderEndpoint for MexcSpotApiClientWithAuthentication {
    async fn cancel_order(&self, params: CancelOrderParams<'_>) -> ApiResult<CancelOrderOutput> {
        let endpoint = format!("{}/api/v3/order", self.endpoint.as_ref());
        let query = CancelOrderQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self.reqwest_client.delete(&endpoint).query(&query_with_signature).send().await?;
        let api_response = response.json::<ApiResponse<CancelOrderOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancel_order() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let params = CancelOrderParams {
            symbol: "KASUSDT",
            order_id: None,
            original_client_order_id: Some("MY_ORDER_ID"),
            new_client_order_id: None,
        };
        let result = client.cancel_order(params).await;
        assert!(result.is_ok());
    }
}
