use crate::futures::auth::SignRequestParamsKind;
use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::{OpenType, OrderSide, OrderType, PositionMode};
use crate::futures::MexcFuturesApiClientWithAuthentication;
use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct OrderParams<'a> {
    pub symbol: &'a str,
    pub price: Decimal,
    pub volume: Decimal,
    pub leverage: Option<u32>,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub open_type: OpenType,
    pub position_id: Option<i64>,
    pub external_order_id: Option<&'a str>,
    pub stop_loss_price: Option<Decimal>,
    pub take_profit_price: Option<Decimal>,
    pub position_mode: Option<PositionMode>,
    pub reduce_only: Option<bool>,
}

#[derive(Debug)]
pub struct OrderOutput {
    pub order_id: i64,
}

#[async_trait]
pub trait Order {
    async fn order<'a>(&self, params: OrderParams<'a>) -> ApiResult<OrderOutput>;
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderPayload<'a> {
    pub symbol: &'a str,
    pub price: Decimal,
    #[serde(rename = "vol")]
    pub volume: Decimal,
    pub leverage: Option<u32>,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub open_type: OpenType,
    pub position_id: Option<i64>,
    #[serde(rename = "externalOid")]
    pub external_order_id: Option<&'a str>,
    pub stop_loss_price: Option<Decimal>,
    pub take_profit_price: Option<Decimal>,
    pub position_mode: Option<PositionMode>,
    pub reduce_only: Option<bool>,
}

impl<'a> From<&OrderParams<'a>> for OrderPayload<'a> {
    fn from(params: &OrderParams<'a>) -> Self {
        OrderPayload {
            symbol: params.symbol,
            price: params.price,
            volume: params.volume,
            leverage: params.leverage,
            side: params.side,
            order_type: params.order_type,
            open_type: params.open_type,
            position_id: params.position_id,
            external_order_id: params.external_order_id,
            stop_loss_price: params.stop_loss_price,
            take_profit_price: params.take_profit_price,
            position_mode: params.position_mode,
            reduce_only: params.reduce_only,
        }
    }
}

#[async_trait]
impl Order for MexcFuturesApiClientWithAuthentication {
    async fn order<'a>(&self, params: OrderParams<'a>) -> ApiResult<OrderOutput> {
        let url = format!("{}/api/v1/private/order/submit", self.endpoint.as_ref());
        let payload = OrderPayload::from(&params);
        let auth_header_map = self.get_auth_header_map(&payload, SignRequestParamsKind::Body)?;
        let response = self
            .reqwest_client
            .post(&url)
            .headers(auth_header_map)
            .json(&payload)
            .send()
            .await?;
        let api_response = response.json::<ApiResponse<i64>>().await?;
        let order_id = api_response.into_api_result()?;

        Ok(OrderOutput { order_id })
    }
}
