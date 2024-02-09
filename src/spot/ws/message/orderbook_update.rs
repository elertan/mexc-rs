use rust_decimal::Decimal;

use super::{RawChannelMessage, RawChannelMessageData, RawEventChannelMessageData};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawOrderData {
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "v")]
    pub quantity: Decimal,
}

#[derive(Debug, Clone)]
pub struct OrderUpdateMessage {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug)]
pub struct OrderbookUpdateMessage {
    pub symbol: String,
    pub version: u128,
    pub asks: Vec<OrderUpdateMessage>,
    pub bids: Vec<OrderUpdateMessage>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelMessageToOrderbookUpdateMessageError {
    #[error("No orderbook update message")]
    NoOrderbookUpdateMessage,
}

pub(crate) fn channel_message_to_spot_orderbook_update_message(
    channel_message: &RawChannelMessage,
) -> Result<OrderbookUpdateMessage, ChannelMessageToOrderbookUpdateMessageError> {
    let Some(symbol) = &channel_message.symbol else {
        return Err(ChannelMessageToOrderbookUpdateMessageError::NoOrderbookUpdateMessage);
    };
    let RawChannelMessageData::Event(event) = &channel_message.data else {
        return Err(ChannelMessageToOrderbookUpdateMessageError::NoOrderbookUpdateMessage);
    };
    let RawEventChannelMessageData::OrdersUpdate{ asks, bids, version, .. } = &event else {
        return Err(ChannelMessageToOrderbookUpdateMessageError::NoOrderbookUpdateMessage);
    };

    let message = OrderbookUpdateMessage {
        symbol: symbol.clone(),
        version: version
            .parse()
            .map_err(|_| ChannelMessageToOrderbookUpdateMessageError::NoOrderbookUpdateMessage)?,
        asks: match asks {
            Some(asks) => asks
                .iter()
                .map(|raw| OrderUpdateMessage {
                    price: raw.price,
                    quantity: raw.quantity,
                })
                .collect(),
            None => vec![],
        },
        bids: match bids {
            Some(bids) => bids
                .iter()
                .map(|raw| OrderUpdateMessage {
                    price: raw.price,
                    quantity: raw.quantity,
                })
                .collect(),
            None => vec![],
        },
    };
    Ok(message)
}
