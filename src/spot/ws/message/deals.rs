use crate::spot::ws::message::{
    RawChannelMessage, RawChannelMessageData, RawEventChannelMessageData,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawSpotDealData {
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "v")]
    pub quantity: Decimal,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "S")]
    pub trade_type: i32,
}

#[derive(Debug)]
pub struct SpotDealsMessage {
    pub deals: Vec<SpotDeal>,
}

#[derive(Debug)]
pub struct SpotDeal {
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
    pub trade_type: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelMessageToSpotDealsMessageError {
    #[error("No deals message")]
    NoDealsMessage,
}

pub(crate) fn channel_message_to_spot_deals_message(
    channel_message: &RawChannelMessage,
) -> Result<SpotDealsMessage, ChannelMessageToSpotDealsMessageError> {
    let Some(symbol) = &channel_message.symbol else {
        return Err(ChannelMessageToSpotDealsMessageError::NoDealsMessage);
    };
    let RawChannelMessageData::Event(event) = &channel_message.data else {
        return Err(ChannelMessageToSpotDealsMessageError::NoDealsMessage);
    };
    let RawEventChannelMessageData::Deals { deals, .. } = &event else {
        return Err(ChannelMessageToSpotDealsMessageError::NoDealsMessage);
    };

    let spot_deals = deals
        .iter()
        .map(|deal| SpotDeal {
            symbol: symbol.clone(),
            price: deal.price,
            quantity: deal.quantity,
            timestamp: deal.timestamp,
            trade_type: deal.trade_type,
        })
        .collect();

    let message = SpotDealsMessage { deals: spot_deals };
    Ok(message)
}
