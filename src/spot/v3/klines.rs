use crate::spot::v3::enums::KlineInterval;
use crate::spot::v3::{ApiError, ApiResult, ErrorResponse};
use crate::spot::{MexcSpotApiClient, MexcSpotApiClientWithAuthentication, MexcSpotApiEndpoint};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlinesParams<'a> {
    /// Symbol
    pub symbol: &'a str,
    /// Interval
    pub interval: KlineInterval,
    /// Start time
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
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
    // #[serde(deserialize_with = "volume_decimal")]
    pub volume: Decimal,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub close_time: DateTime<Utc>,
    pub quote_asset_volume: Decimal,
}

// fn volume_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let v = Value::deserialize(deserializer)?;
//     let Ok(deser) = <Decimal as Deserialize>::deserialize(v.clone()) else {
//         tracing::error!("Unable to parse Decimal for volume: {}", v);
//         return Ok(Decimal::from(0));
//     };
//     Ok(deser)
// }

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

    #[allow(clippy::get_first)]
    let klines = kline_values
        .into_iter()
        .map(|kline_value| {
            let serde_json::Value::Array(entries) = kline_value else {
                return Err(ApiError::UnableToParseResponse);
            };

            let open_time_ts_milliseconds = entries
                .get(0)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_i64()
                .ok_or(ApiError::UnableToParseResponse)?;
            let open_time = Utc.timestamp_millis_opt(open_time_ts_milliseconds).unwrap();

            let open_str = entries
                .get(1)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let open_value = filter_decimal_str(open_str);
            let open = open_value.parse::<Decimal>().map_err(|err| {
                tracing::error!("Unable to parse Decimal for open: {}", err);
                ApiError::UnableToParseResponse
            })?;

            let high_str = entries
                .get(2)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let high_value = filter_decimal_str(high_str);
            let high = high_value.parse::<Decimal>().map_err(|err| {
                tracing::error!("Unable to parse Decimal for high: {}", err);
                ApiError::UnableToParseResponse
            })?;

            let low_str = entries
                .get(3)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let low_value = filter_decimal_str(low_str);
            let low = low_value.parse::<Decimal>().map_err(|err| {
                tracing::error!("Unable to parse Decimal for low: {}", err);
                ApiError::UnableToParseResponse
            })?;

            let close_str = entries
                .get(4)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let close_value = filter_decimal_str(close_str);
            let close = close_value.parse::<Decimal>().map_err(|err| {
                tracing::error!("Unable to parse Decimal for close: {}", err);
                ApiError::UnableToParseResponse
            })?;

            let volume_str = entries
                .get(5)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let volume_value = filter_decimal_str(volume_str);
            let volume = volume_value.parse::<Decimal>().map_err(|err| {
                tracing::error!("Unable to parse Decimal for volume: {}", err,);
                ApiError::UnableToParseResponse
            })?;

            let close_time_ts_milliseconds = entries
                .get(6)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_i64()
                .ok_or(ApiError::UnableToParseResponse)?;
            let close_time = Utc
                .timestamp_millis_opt(close_time_ts_milliseconds)
                .unwrap();

            let quote_asset_volume_str = entries
                .get(7)
                .ok_or(ApiError::UnableToParseResponse)?
                .as_str()
                .ok_or(ApiError::UnableToParseResponse)?;
            let quote_asset_volume_value = filter_decimal_str(quote_asset_volume_str);
            let quote_asset_volume =
                quote_asset_volume_value.parse::<Decimal>().map_err(|err| {
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

fn filter_decimal_str(string: &str) -> String {
    string
        .chars()
        .filter(|c| c.is_numeric() || c == &'.')
        .collect::<String>()
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
    use chrono::Duration;

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

    #[tokio::test]
    async fn test_klines_with_time_range() {
        let start_time = Utc.with_ymd_and_hms(2023, 9, 1, 0, 0, 0).unwrap();
        let end_time = start_time + Duration::minutes(1000);
        eprintln!("start_time: {}", start_time);
        eprintln!("end_time: {}", end_time);

        // BTCUSDT&interval=1m&startTime=1695560040000&endTime=1695560940000
        // BTCUSDT&interval=1m&startTime=1598918400000&endTime=1598978400000

        let client = MexcSpotApiClient::default();
        let params = KlinesParams {
            symbol: "BTCUSDT",
            interval: KlineInterval::OneMinute,
            start_time: Some(start_time),
            end_time: Some(end_time),
            limit: Some(1000),
        };
        let result = client.klines(params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        eprintln!("len: {}", output.klines.len());

        let first_kline = output.klines.first().unwrap();
        eprintln!("first kline time: {}", first_kline.open_time);

        let last_kline = output.klines.last().unwrap();
        eprintln!("last kline time: {}", last_kline.close_time);
    }
}
