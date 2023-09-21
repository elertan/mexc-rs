use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    pub currency: String,
    pub position_margin: BigDecimal,
    pub frozen_balance: BigDecimal,
    pub available_balance: BigDecimal,
    pub cash_balance: BigDecimal,
    pub equity: BigDecimal,
    pub unrealized: BigDecimal,
    pub bonus: Option<BigDecimal>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPosition {
    pub position_id: i64,
    pub symbol: String,
    #[serde(rename = "holdVol")]
    pub holding_volume: BigDecimal,
    pub position_type: PositionType,
    pub open_type: OpenType,
    pub state: PositionState,
    #[serde(rename = "frozenVol")]
    pub frozen_volume: BigDecimal,
    #[serde(rename = "closeVol")]
    pub close_volume: BigDecimal,
    #[serde(rename = "holdAvgPrice")]
    pub holdings_average_price: BigDecimal,
    #[serde(rename = "closeAvgPrice")]
    pub close_average_price: BigDecimal,
    #[serde(rename = "openAvgPrice")]
    pub open_average_price: BigDecimal,
    pub liquidate_price: BigDecimal,
    #[serde(rename = "oim")]
    pub original_initial_margin: BigDecimal,
    pub adl_level: Option<i8>,
    #[serde(rename = "im")]
    pub initial_margin: BigDecimal,
    pub hold_fee: BigDecimal,
    pub realised: BigDecimal,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i8)]
pub enum PositionType {
    Long = 1,
    Short = 2,
}

#[derive(Debug, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i8)]
pub enum OpenType {
    Isolated = 1,
    Cross = 2,
}

#[derive(Debug, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i8)]
pub enum PositionState {
    Holding = 1,
    SystemAutoHolding = 2,
    Closed = 3,
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

        let open_position: OpenPosition = serde_json::from_str(json).unwrap();
    }
}
