use crate::spot::v3::models::Order;
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct GetOpenOrdersParams<'a> {
    pub symbol: &'a str,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderQuery<'a> {
    pub symbol: &'a str,
    /// Max 60000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<GetOpenOrdersParams<'a>> for GetOrderQuery<'a> {
    fn from(params: GetOpenOrdersParams<'a>) -> Self {
        Self {
            symbol: params.symbol,
            recv_window: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct GetOrderOutput {
    pub orders: Vec<Order>,
}

#[async_trait]
pub trait GetOpenOrdersEndpoint {
    async fn get_open_orders(&self, params: GetOpenOrdersParams<'_>) -> ApiResult<GetOrderOutput>;
}

#[async_trait]
impl GetOpenOrdersEndpoint for MexcSpotApiClientWithAuthentication {
    async fn get_open_orders(&self, params: GetOpenOrdersParams<'_>) -> ApiResult<GetOrderOutput> {
        let endpoint = format!("{}/api/v3/openOrders", self.endpoint.as_ref());
        let query = GetOrderQuery::from(params);
        let query_with_signature = self.sign_query(query)?;

        let response = self
            .reqwest_client
            .get(&endpoint)
            .query(&query_with_signature)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<Vec<Order>>>().await?;
        let orders = api_response.into_api_result()?;

        let output = GetOrderOutput { orders };

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_open_orders() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let params = GetOpenOrdersParams { symbol: "KASUSDT" };
        let result = client.get_open_orders(params).await;
        eprintln!("{:?}", &result);
        assert!(result.is_ok());
    }

    #[test]
    fn deser() {
        let j = r#"[{"symbol":"KASUSDT","orderId":"C01__334660835685203969","orderListId":-1,"clientOrderId":"","price":"0.04","origQty":"183.65","executedQty":"0","cummulativeQuoteQty":"0","status":"NEW","timeInForce":null,"type":"LIMIT","side":"BUY","stopPrice":null,"icebergQty":null,"time":1695571596791,"updateTime":null,"isWorking":true,"origQuoteOrderQty":"7.346"}]"#;

        let deserializer = &mut serde_json::Deserializer::from_str(j);

        let result: Result<Vec<Order>, _> = serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", &result);
    }
}
