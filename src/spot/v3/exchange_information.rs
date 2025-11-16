use crate::spot::v3::enums::OrderType;
use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiTrait;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum ExchangeInformationParams<'a> {
    None,
    Symbol(&'a str),
    Symbols(&'a [&'a str]),
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformationSymbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: i32,
    pub quote_asset: String,
    pub quote_precision: i32,
    pub quote_asset_precision: i32,
    pub base_commission_precision: i32,
    pub quote_commission_precision: i32,
    pub order_types: Vec<OrderType>,
    pub quote_order_qty_market_allowed: Option<bool>,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub quote_amount_precision: Decimal,
    pub base_size_precision: Decimal,
    pub permissions: Vec<String>,
    pub filters: Vec<serde_json::Value>,
    pub max_quote_amount: Decimal,
    pub maker_commission: Decimal,
    pub taker_commission: Decimal,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformationOutput {
    pub timezone: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub server_time: DateTime<Utc>,
    pub rate_limits: Vec<serde_json::Value>,
    pub exchange_filters: Vec<serde_json::Value>,
    pub symbols: Vec<ExchangeInformationSymbol>,
}

#[derive(Debug, serde::Serialize)]
pub struct ExchangeInformationEndpointQueryParams<'a> {
    pub symbol: Option<&'a str>,
    pub symbols: Option<String>,
}

impl<'a> From<ExchangeInformationParams<'a>> for ExchangeInformationEndpointQueryParams<'a> {
    fn from(value: ExchangeInformationParams<'a>) -> Self {
        match value {
            ExchangeInformationParams::None => Self {
                symbol: None,
                symbols: None,
            },
            ExchangeInformationParams::Symbol(symbol) => Self {
                symbol: Some(symbol),
                symbols: None,
            },
            ExchangeInformationParams::Symbols(symbols) => Self {
                symbol: None,
                symbols: Some(symbols.join(",")),
            },
        }
    }
}

#[async_trait]
pub trait ExchangeInformationEndpoint {
    async fn exchange_information(
        &self,
        params: ExchangeInformationParams<'_>,
    ) -> ApiResult<ExchangeInformationOutput>;
}

#[async_trait]
impl<T: MexcSpotApiTrait + Sync> ExchangeInformationEndpoint for T {
    async fn exchange_information(
        &self,
        params: ExchangeInformationParams<'_>,
    ) -> ApiResult<ExchangeInformationOutput> {
        let endpoint = format!("{}/api/v3/exchangeInfo", self.endpoint().as_ref());
        let query_params = ExchangeInformationEndpointQueryParams::from(params);
        let response = self
            .reqwest_client()
            .get(&endpoint)
            .query(&query_params)
            .send()
            .await?;
        let api_response = response
            .json::<ApiResponse<ExchangeInformationOutput>>()
            .await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::spot::MexcSpotApiClient;

    use super::*;

    #[tokio::test]
    async fn test_no_params() {
        let client = MexcSpotApiClient::default();
        let params = ExchangeInformationParams::None;
        let result = client.exchange_information(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_single_symbol() {
        let client = MexcSpotApiClient::default();
        let params = ExchangeInformationParams::Symbol("BTCUSDT");
        let result = client.exchange_information(params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.symbols.len(), 1);
        let first_symbol = &output.symbols[0];
        assert_eq!(first_symbol.symbol, "BTCUSDT");
    }

    #[tokio::test]
    async fn test_multiple_symbols() {
        let client = MexcSpotApiClient::default();
        let params = ExchangeInformationParams::Symbols(&["BTCUSDT", "ETHUSDT"]);
        let result = client.exchange_information(params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.symbols.len(), 2);
    }
}
