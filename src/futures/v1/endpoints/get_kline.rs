use crate::futures::response::ApiResponse;
use crate::futures::result::ApiResult;
use crate::futures::v1::models::{Kline, KlineInterval};
use crate::futures::{
    MexcFuturesApiClient, MexcFuturesApiClientWithAuthentication, MexcFuturesApiEndpoint,
};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct GetKlineParams<'a> {
    pub symbol: &'a str,
    pub interval: KlineInterval,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct GetKlineQuery {
    pub interval: KlineInterval,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub start: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end: Option<DateTime<Utc>>,
}

impl From<GetKlineParams<'_>> for GetKlineQuery {
    fn from(params: GetKlineParams<'_>) -> Self {
        Self {
            interval: params.interval,
            start: params.start,
            end: params.end,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct KlineData {
    pub time: Vec<i64>,
    pub open: Vec<Decimal>,
    pub close: Vec<Decimal>,
    pub high: Vec<Decimal>,
    pub low: Vec<Decimal>,
    pub vol: Vec<Decimal>,
    pub amount: Vec<Decimal>,
}

#[derive(Debug)]
pub struct GetKlineOutput {
    pub klines: Vec<Kline>,
}

#[async_trait]
pub trait GetKline {
    async fn get_kline(&self, params: GetKlineParams<'_>) -> ApiResult<GetKlineOutput>;
}

async fn default_impl(
    endpoint: &MexcFuturesApiEndpoint,
    reqwest: &Client,
    params: GetKlineParams<'_>,
) -> ApiResult<GetKlineOutput> {
    let url = format!(
        "{}/api/v1/contract/kline/{}",
        endpoint.as_ref(),
        params.symbol
    );
    let query = GetKlineQuery::from(params);
    let response = reqwest.get(&url).query(&query).send().await?;
    let api_response = response.json::<ApiResponse<KlineData>>().await?;
    let data = api_response.into_api_result()?;

    let amount_of_entries = data.time.len();
    let mut klines = Vec::with_capacity(amount_of_entries);
    for i in 0..amount_of_entries {
        let kline = Kline {
            time: Utc.timestamp_opt(data.time[i], 0).unwrap(),
            open: data.open[i],
            close: data.close[i],
            high: data.high[i],
            low: data.low[i],
            volume: data.vol[i],
            amount: data.amount[i],
        };
        klines.push(kline);
    }

    Ok(GetKlineOutput { klines })
}

#[async_trait]
impl GetKline for MexcFuturesApiClient {
    async fn get_kline(&self, params: GetKlineParams<'_>) -> ApiResult<GetKlineOutput> {
        default_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}

#[async_trait]
impl GetKline for MexcFuturesApiClientWithAuthentication {
    async fn get_kline(&self, params: GetKlineParams<'_>) -> ApiResult<GetKlineOutput> {
        default_impl(&self.endpoint, &self.reqwest_client, params).await
    }
}
