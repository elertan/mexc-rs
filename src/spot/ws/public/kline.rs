use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use crate::spot::v3::enums::KlineInterval;
use crate::spot::ws::public::PublicChannelMessage;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineIntervalSubscription {
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

impl AsRef<str> for KlineIntervalSubscription {
    fn as_ref(&self) -> &str {
        match self {
            KlineIntervalSubscription::OneMinute => "Min1",
            KlineIntervalSubscription::FiveMinutes => "Min5",
            KlineIntervalSubscription::FifteenMinutes => "Min15",
            KlineIntervalSubscription::ThirtyMinutes => "Min30",
            KlineIntervalSubscription::OneHour => "Min60",
            KlineIntervalSubscription::FourHours => "Hour4",
            KlineIntervalSubscription::OneDay => "Day1",
            KlineIntervalSubscription::OneMonth => "Month1",
        }
    }
}

impl From<KlineIntervalSubscription> for KlineInterval {
    fn from(value: KlineIntervalSubscription) -> Self {
        match value {
            KlineIntervalSubscription::OneMinute => KlineInterval::OneMinute,
            KlineIntervalSubscription::FiveMinutes => KlineInterval::FiveMinutes,
            KlineIntervalSubscription::FifteenMinutes => KlineInterval::FifteenMinutes,
            KlineIntervalSubscription::ThirtyMinutes => KlineInterval::ThirtyMinutes,
            KlineIntervalSubscription::OneHour => KlineInterval::OneHour,
            KlineIntervalSubscription::FourHours => KlineInterval::FourHours,
            KlineIntervalSubscription::OneDay => KlineInterval::OneDay,
            KlineIntervalSubscription::OneMonth => KlineInterval::OneMonth,
        }
    }
}

impl From<KlineInterval> for KlineIntervalSubscription {
    fn from(value: KlineInterval) -> Self {
        match value {
            KlineInterval::OneMinute => KlineIntervalSubscription::OneMinute,
            KlineInterval::FiveMinutes => KlineIntervalSubscription::FiveMinutes,
            KlineInterval::FifteenMinutes => KlineIntervalSubscription::FifteenMinutes,
            KlineInterval::ThirtyMinutes => KlineIntervalSubscription::ThirtyMinutes,
            KlineInterval::OneHour => KlineIntervalSubscription::OneHour,
            KlineInterval::FourHours => KlineIntervalSubscription::FourHours,
            KlineInterval::OneDay => KlineIntervalSubscription::OneDay,
            KlineInterval::OneMonth => KlineIntervalSubscription::OneMonth,
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
    pub interval: KlineIntervalSubscription,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelMessageToSpotKlineMessageError {
    #[error("No kline message")]
    NoKlineMessage,
}

pub(crate) fn channel_message_to_spot_kline_message(channel_message: &PublicChannelMessage) -> Result<SpotKlineMessage, ChannelMessageToSpotKlineMessageError> {
    let Some(symbol) = &channel_message.symbol else {
        return Err(ChannelMessageToSpotKlineMessageError::NoKlineMessage);
    };
    let Some(kline) = &channel_message.data.k else {
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
