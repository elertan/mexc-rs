use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    pub currency: String,
    pub position_margin: Decimal,
    pub frozen_balance: Decimal,
    pub available_balance: Decimal,
    pub cash_balance: Decimal,
    pub equity: Decimal,
    pub unrealized: Decimal,
    pub bonus: Option<Decimal>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPosition {
    pub position_id: i64,
    pub symbol: String,
    #[serde(rename = "holdVol")]
    pub holding_volume: Decimal,
    pub position_type: PositionType,
    pub open_type: OpenType,
    pub state: PositionState,
    #[serde(rename = "frozenVol")]
    pub frozen_volume: Decimal,
    #[serde(rename = "closeVol")]
    pub close_volume: Decimal,
    #[serde(rename = "holdAvgPrice")]
    pub holdings_average_price: Decimal,
    #[serde(rename = "closeAvgPrice")]
    pub close_average_price: Decimal,
    #[serde(rename = "openAvgPrice")]
    pub open_average_price: Decimal,
    pub liquidate_price: Decimal,
    #[serde(rename = "oim")]
    pub original_initial_margin: Decimal,
    pub adl_level: Option<i8>,
    #[serde(rename = "im")]
    pub initial_margin: Decimal,
    pub hold_fee: Decimal,
    pub realised: Decimal,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum PositionType {
    Long = 1,
    Short = 2,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OpenType {
    Isolated = 1,
    Cross = 2,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum PositionState {
    Holding = 1,
    SystemAutoHolding = 2,
    Closed = 3,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub order_id: i64,
    pub symbol: String,
    pub position_id: i64,
    #[serde(rename = "price")]
    pub trigger_price: Decimal,
    #[serde(rename = "vol")]
    pub trigger_volume: Decimal,
    pub leverage: i32,
    pub side: OrderSide,
    pub category: OrderCategory,
    pub order_type: OrderType,
    #[serde(rename = "dealAvgPrice")]
    pub deal_average_price: Decimal,
    #[serde(rename = "dealVol")]
    pub deal_volume: Decimal,
    pub order_margin: Decimal,
    pub used_margin: Decimal,
    pub taker_fee: Decimal,
    pub maker_fee: Decimal,
    pub profit: Decimal,
    pub fee_currency: String,
    pub open_type: OpenType,
    pub state: OrderState,
    pub error_code: OrderErrorCode,
    #[serde(rename = "externalOid")]
    pub external_order_id: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub update_time: Option<DateTime<Utc>>,
    pub stop_loss_price: Option<Decimal>,
    pub take_profit_price: Option<Decimal>,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OrderSide {
    OpenLong = 1,
    CloseShort = 2,
    OpenShort = 3,
    CloseLong = 4,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OrderCategory {
    LimitOrder = 1,
    SystemTakeOverDelegate = 2,
    CloseDelegate = 3,
    ADLReduction = 4,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OrderType {
    PriceLimitedOrder = 1,
    PostOnlyMaker = 2,
    TransactOrCancelInstantly = 3,
    TransactCompletelyOrCancelCompletely = 4,
    MarketOrders = 5,
    ConvertMarketPriceToCurrentPrice = 6,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OrderState {
    Uninformed = 1,
    Uncompleted = 2,
    Completed = 3,
    Cancelled = 4,
    Invalid = 5,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum OrderErrorCode {
    Normal = 0,
    ParameterErrors = 1,
    AccountBalanceIsInsufficient = 2,
    ThePositionDoesNotExist = 3,
    PositionInsufficient = 4,
    ForLongPositionsTheOrderPriceIsLessThanTheClosePriceWhileForShortPositionsTheOrderPriceIsGreaterThanTheClosePrice =
        5,
    WhenOpeningLongTheClosePriceIsMoreThanTheFairPriceWhileWhenOpeningShortTheClosePriceIsLessThanTheFairPrice =
        6,
    ExceedRiskQuotaRestrictions = 7,
    SystemCancelled = 8,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Copy,
    Clone,
)]
#[repr(i8)]
pub enum PositionMode {
    Hedge = 1,
    OneWay = 2,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineInterval {
    /// 1 minute
    #[serde(rename = "Min1")]
    OneMinute,

    /// 5 minutes
    #[serde(rename = "Min5")]
    FiveMinutes,

    /// 15 minutes
    #[serde(rename = "Min15")]
    FifteenMinutes,

    /// 30 minutes
    #[serde(rename = "Min30")]
    ThirtyMinutes,

    /// 1 hour
    #[serde(rename = "Min60")]
    OneHour,

    /// 4 hours
    #[serde(rename = "Hour4")]
    FourHours,

    /// 8 hours
    #[serde(rename = "Hour8")]
    EightHours,

    /// 1 day
    #[serde(rename = "Day1")]
    OneDay,

    /// 1 week
    #[serde(rename = "Week1")]
    OneWeek,

    /// 1 month
    #[serde(rename = "Month1")]
    OneMonth,
}

#[derive(Debug)]
pub struct Kline {
    pub time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub amount: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_open_position() {
        let json = r#"
        {
            "positionId": 1394650,
            "symbol": "ETH_USDT",
            "positionType": 1,
            "openType": 1,
            "state": 1,
            "holdVol": 1,
            "frozenVol": 0,
            "closeVol": 0,
            "holdAvgPrice": 1217.3,
            "openAvgPrice": 1217.3,
            "closeAvgPrice": 0,
            "liquidatePrice": 1211.2,
            "oim": 0.1290338,
            "im": 0.1290338,
            "holdFee": 0,
            "realised": -0.0073,
            "leverage": 100,
            "createTime": 1609991676000,
            "updateTime": 1609991676000,
            "autoAddIm": false
        }
        "#;

        let _open_position: OpenPosition = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn deserialize_open_order() {
        let json = r#"
        {
            "orderId": 0,
            "symbol": "",
            "positionId": 0,
            "price": 0.0,
            "vol": 0.0,
            "leverage": 0,
            "side": 1,
            "category": 1,
            "orderType": 1,
            "dealAvgPrice": 0.0,
            "dealVol": 0.0,
            "orderMargin": 0.0,
            "takerFee": 0.0,
            "makerFee": 0.0,
            "profit": 0.0,
            "feeCurrency": "",
            "openType": 1,
            "state": 1,
            "externalOid": "",
            "errorCode": 0,
            "usedMargin": 0.0,
            "createTime": 1609991676000,
            "updateTime": 1609991676000,
            "stopLossPrice": 0.0,
            "takeProfitPrice": 0.0
        }
        "#;

        let _open_order: OpenOrder = serde_json::from_str(json).unwrap();
    }
}
