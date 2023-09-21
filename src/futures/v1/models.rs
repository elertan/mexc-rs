use bigdecimal::BigDecimal;

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
}
