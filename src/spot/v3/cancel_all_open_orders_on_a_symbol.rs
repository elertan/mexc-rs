use crate::spot::v3::enums::{OrderSide, OrderStatus, OrderType};
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct CancelAllOpenOrdersOnASymbolParams<'a> {
    pub symbol: &'a str,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllOpenOrdersOnASymbolQuery<'a> {
    pub symbol: &'a str,
    /// Max 60000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<CancelAllOpenOrdersOnASymbolParams<'a>> for CancelAllOpenOrdersOnASymbolQuery<'a> {
    fn from(params: CancelAllOpenOrdersOnASymbolParams<'a>) -> Self {
        Self {
            symbol: params.symbol,
            recv_window: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct CancelAllOpenOrdersOnASymbolOutput {
    pub canceled_orders: Vec<CanceledOrder>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanceledOrder {
    pub symbol: String,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub price: Decimal,
    #[serde(rename = "origQty")]
    pub original_quantity: Decimal,
    #[serde(rename = "executedQty")]
    pub executed_quantity: Decimal,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cummulative_quote_quantity: Decimal,
    pub status: OrderStatus,
    pub time_in_force: Option<String>,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
}

#[async_trait]
pub trait CancelAllOpenOrdersOnASymbolEndpoint {
    async fn cancel_all_open_orders_on_a_symbol(
        &self,
        params: CancelAllOpenOrdersOnASymbolParams<'_>,
    ) -> ApiResult<CancelAllOpenOrdersOnASymbolOutput>;
}

#[async_trait]
impl CancelAllOpenOrdersOnASymbolEndpoint for MexcSpotApiClientWithAuthentication {
    async fn cancel_all_open_orders_on_a_symbol(
        &self,
        params: CancelAllOpenOrdersOnASymbolParams<'_>,
    ) -> ApiResult<CancelAllOpenOrdersOnASymbolOutput> {
        let endpoint = format!("{}/api/v3/openOrders", self.endpoint.as_ref());
        let query = CancelAllOpenOrdersOnASymbolQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self
            .reqwest_client
            .delete(&endpoint)
            .query(&query_with_signature)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<Vec<CanceledOrder>>>().await?;
        let canceled_orders = api_response.into_api_result()?;

        Ok(CancelAllOpenOrdersOnASymbolOutput { canceled_orders })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancel_order() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let params = CancelAllOpenOrdersOnASymbolParams { symbol: "KASUSDT" };
        let result = client.cancel_all_open_orders_on_a_symbol(params).await;
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize() {
        let json = r#"
            [{"symbol":"KASUSDT","orderId":"C01__333180898079965185","price":"0.001","origQty":"5000","type":"LIMIT","side":"BUY","executedQty":"0","cummulativeQuoteQty":"0","status":"NEW"}]
        "#;
        // serde path to error
        let _canceled_orders = serde_json::from_str::<Vec<CanceledOrder>>(json).unwrap();
    }
}
