use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use crate::spot::ws::public::PublicChannelMessage;

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

pub(crate) fn channel_message_to_spot_deals_message(channel_message: &PublicChannelMessage) -> Result<SpotDealsMessage, ChannelMessageToSpotDealsMessageError> {
    let Some(deals) = &channel_message.data.deals else {
        return Err(ChannelMessageToSpotDealsMessageError::NoDealsMessage);
    };

    let spot_deals = deals.iter().map(|deal| SpotDeal {
        symbol: channel_message.symbol.clone(),
        price: deal.price.clone(),
        quantity: deal.quantity.clone(),
        timestamp: deal.timestamp.clone(),
        trade_type: deal.trade_type,
    }).collect();

    let message = SpotDealsMessage {
        deals: spot_deals,
    };
    Ok(message)
}
