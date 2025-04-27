use crate::spot::v3::enums::{OrderSide, OrderStatus, OrderType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
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
    pub stop_price: Option<Decimal>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub update_time: Option<DateTime<Utc>>,
    pub is_working: bool,
    #[serde(rename = "origQuoteOrderQty")]
    pub original_quote_order_qty: Decimal,
}
