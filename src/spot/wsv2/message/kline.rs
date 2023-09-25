use crate::spot::v3::enums::KlineInterval;
use crate::spot::wsv2::message::{
    RawChannelMessage, RawChannelMessageData, RawEventChannelMessageData,
    RawEventEventChannelMessageData,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawKlineData {
    #[serde(rename = "T", with = "chrono::serde::ts_seconds")]
    pub T: DateTime<Utc>,
    pub a: Decimal,
    pub c: Decimal,
    pub h: Decimal,
    pub l: Decimal,
    pub o: Decimal,
    pub i: KlineIntervalTopic,
    #[serde(rename = "t", with = "chrono::serde::ts_seconds")]
    pub t: DateTime<Utc>,
    pub v: Decimal,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineIntervalTopic {
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

    /// 1 day
    #[serde(rename = "Day1")]
    OneDay,

    /// 1 month
    #[serde(rename = "Month1")]
    OneMonth,
}

impl AsRef<str> for KlineIntervalTopic {
    fn as_ref(&self) -> &str {
        match self {
            KlineIntervalTopic::OneMinute => "Min1",
            KlineIntervalTopic::FiveMinutes => "Min5",
            KlineIntervalTopic::FifteenMinutes => "Min15",
            KlineIntervalTopic::ThirtyMinutes => "Min30",
            KlineIntervalTopic::OneHour => "Min60",
            KlineIntervalTopic::FourHours => "Hour4",
            KlineIntervalTopic::OneDay => "Day1",
            KlineIntervalTopic::OneMonth => "Month1",
        }
    }
}

impl From<KlineIntervalTopic> for KlineInterval {
    fn from(value: KlineIntervalTopic) -> Self {
        match value {
            KlineIntervalTopic::OneMinute => KlineInterval::OneMinute,
            KlineIntervalTopic::FiveMinutes => KlineInterval::FiveMinutes,
            KlineIntervalTopic::FifteenMinutes => KlineInterval::FifteenMinutes,
            KlineIntervalTopic::ThirtyMinutes => KlineInterval::ThirtyMinutes,
            KlineIntervalTopic::OneHour => KlineInterval::OneHour,
            KlineIntervalTopic::FourHours => KlineInterval::FourHours,
            KlineIntervalTopic::OneDay => KlineInterval::OneDay,
            KlineIntervalTopic::OneMonth => KlineInterval::OneMonth,
        }
    }
}

impl From<KlineInterval> for KlineIntervalTopic {
    fn from(value: KlineInterval) -> Self {
        match value {
            KlineInterval::OneMinute => KlineIntervalTopic::OneMinute,
            KlineInterval::FiveMinutes => KlineIntervalTopic::FiveMinutes,
            KlineInterval::FifteenMinutes => KlineIntervalTopic::FifteenMinutes,
            KlineInterval::ThirtyMinutes => KlineIntervalTopic::ThirtyMinutes,
            KlineInterval::OneHour => KlineIntervalTopic::OneHour,
            KlineInterval::FourHours => KlineIntervalTopic::FourHours,
            KlineInterval::OneDay => KlineIntervalTopic::OneDay,
            KlineInterval::OneMonth => KlineIntervalTopic::OneMonth,
        }
    }
}

#[derive(Debug)]
pub struct SpotKlineMessage {
    pub symbol: String,
    pub volume: Decimal,
    pub close: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub open: Decimal,
    pub quantity: Decimal,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub interval: KlineIntervalTopic,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelMessageToSpotKlineMessageError {
    #[error("No kline message")]
    NoKlineMessage,
}

pub(crate) fn channel_message_to_spot_kline_message(
    channel_message: &RawChannelMessage,
) -> Result<SpotKlineMessage, ChannelMessageToSpotKlineMessageError> {
    let Some(symbol) = &channel_message.symbol else {
        return Err(ChannelMessageToSpotKlineMessageError::NoKlineMessage);
    };
    let RawChannelMessageData::Event(event) = &channel_message.data else {
        return Err(ChannelMessageToSpotKlineMessageError::NoKlineMessage);
    };
    let RawEventEventChannelMessageData::Kline(kline) = &event.event else {
        return Err(ChannelMessageToSpotKlineMessageError::NoKlineMessage);
    };

    let message = SpotKlineMessage {
        symbol: symbol.clone(),
        interval: kline.i,
        end_time: kline.T,
        volume: kline.a,
        close: kline.c,
        high: kline.h,
        low: kline.l,
        open: kline.o,
        start_time: kline.t,
        quantity: kline.v,
        timestamp: channel_message.timestamp,
    };
    Ok(message)
}
