use async_trait::async_trait;
use rust_decimal::Decimal;
use chrono::{DateTime, TimeZone, Utc};
use crate::spot::{MexcSpotApiClient, MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use crate::spot::v3::{ApiError, ApiResult, ErrorResponse};
use crate::spot::v3::enums::KlineInterval;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlinesParams<'a> {
    /// Symbol
    pub symbol: &'a str,
    /// Interval
    pub interval: KlineInterval,
    /// Start time
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end_time: Option<DateTime<Utc>>,
    /// default 500; max 1000
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlinesOutput {
    pub klines: Vec<Kline>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kline {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub open_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub close_time: DateTime<Utc>,
    pub quote_asset_volume: Decimal,
}

#[async_trait]
pub trait KlinesEndpoint {
    async fn klines(&self, params: KlinesParams<'_>) -> ApiResult<KlinesOutput>;
}

async fn klines_impl(
    endpoint: &MexcSpotApiEndpoint,
    client: &reqwest::Client,
    params: KlinesParams<'_>,
) -> ApiResult<KlinesOutput> {
    let endpoint = format!("{}/api/v3/klines", endpoint.as_ref());
    let response = client.get(&endpoint).query(&params).send().await?;
    let json = response.text().await?;

    if let Ok(err_response) = serde_json::from_str::<ErrorResponse>(&json) {
        return Err(ApiError::ErrorResponse(err_response));
    }
    let value = serde_json::from_str::<serde_json::Value>(&json)?;

    let serde_json::Value::Array(kline_values) = value else {
        return Err(ApiError::UnableToParseResponse);
    };

    let klines = kline_values
        .into_iter()
        .map(|kline_value| {
            let serde_json::Value::Array(entries) = kline_value else {
                return Err(ApiError::UnableToParseResponse);
            };

            let open_time_ts_seconds = entries
                .get(0)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_i64()
                .ok_or(ApiError::UnableToParseResponse)?;
            let open_time = Utc.timestamp_opt(open_time_ts_seconds, 0).unwrap();

            let open = entries
                .get(1)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for open: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let high = entries
                .get(2)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for high: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let low = entries
                .get(3)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for low: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let close = entries
                .get(4)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for close: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let volume = entries
                .get(5)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for volume: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let close_time_ts_seconds = entries
                .get(6)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_i64()
                .ok_or(ApiError::UnableToParseResponse)?;
            let close_time = Utc.timestamp_opt(close_time_ts_seconds, 0).unwrap();

            let quote_asset_volume = entries
                .get(7)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?
                .parse::<Decimal>()
                .map_err(|err| {
                    tracing::error!("Unable to parse Decimal for quote_asset_volume: {}", err);
                    ApiError::UnableToParseResponse
                })?;

            let kline = Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                close_time,
                quote_asset_volume,
            };

            Ok(kline)
        })
        .collect::<Result<_, ApiError>>()?;

    let output = KlinesOutput { klines };

    Ok(output)
}

#[async_trait]
impl KlinesEndpoint for MexcSpotApiClient {
    async fn klines(&self, params: KlinesParams<'_>) -> ApiResult<KlinesOutput> {
        klines_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}

#[async_trait]
impl KlinesEndpoint for MexcSpotApiClientWithAuthentication {
    async fn klines(&self, params: KlinesParams<'_>) -> ApiResult<KlinesOutput> {
        klines_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_klines() {
        let client = MexcSpotApiClient::default();
        let params = KlinesParams {
            symbol: "BTCUSDT",
            interval: KlineInterval::OneMinute,
            start_time: None,
            end_time: None,
            limit: None,
        };
        let result = client.klines(params).await;
        assert!(result.is_ok());
    }
}
