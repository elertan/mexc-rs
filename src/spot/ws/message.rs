use crate::spot::ws::message::account_deals::{
    channel_message_to_account_deals_message, AccountDealsMessage, RawAccountDealsData,
};
use crate::spot::ws::message::account_orders::{
    channel_message_to_account_orders_message, AccountOrdersMessage,
    RawAccountOrdersChannelMessageData,
};
use crate::spot::ws::message::account_update::{
    channel_message_to_account_update_message, AccountUpdateMessage, RawAccountUpdateData,
};
use crate::spot::ws::message::deals::{
    channel_message_to_spot_deals_message, RawSpotDealData, SpotDealsMessage,
};
use crate::spot::ws::message::kline::{
    channel_message_to_spot_kline_message, RawKlineData, SpotKlineMessage,
};
use chrono::{DateTime, Utc};

use self::orderbook_update::{
    channel_message_to_spot_orderbook_update_message, OrderbookUpdateMessage, RawOrderData,
};

pub mod account_deals;
pub mod account_orders;
pub mod account_update;
pub mod deals;
pub mod kline;
pub mod orderbook_update;

#[derive(Debug)]
pub enum Message {
    AccountDeals(AccountDealsMessage),
    AccountUpdate(AccountUpdateMessage),
    AccountOrders(AccountOrdersMessage),
    Deals(SpotDealsMessage),
    Kline(SpotKlineMessage),
    OrderbookUpdate(OrderbookUpdateMessage),
}

impl TryFrom<&RawMessage> for Message {
    type Error = ();

    fn try_from(value: &RawMessage) -> Result<Self, Self::Error> {
        match value {
            RawMessage::IdCodeMessage(_) => Err(()),
            RawMessage::ChannelMessage(raw_channel_message) => match &raw_channel_message.data {
                RawChannelMessageData::AccountDeals(_) => Ok(Message::AccountDeals(
                    channel_message_to_account_deals_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::AccountUpdate(_) => Ok(Message::AccountUpdate(
                    channel_message_to_account_update_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::AccountOrders(_) => Ok(Message::AccountOrders(
                    channel_message_to_account_orders_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::Event(event) => match &event {
                    RawEventChannelMessageData::Deals { .. } => Ok(Message::Deals(
                        channel_message_to_spot_deals_message(raw_channel_message)
                            .map_err(|_| ())?,
                    )),
                    RawEventChannelMessageData::Kline { .. } => Ok(Message::Kline(
                        channel_message_to_spot_kline_message(raw_channel_message)
                            .map_err(|_| ())?,
                    )),
                    RawEventChannelMessageData::OrdersUpdate { .. } => {
                        Ok(Message::OrderbookUpdate(
                            channel_message_to_spot_orderbook_update_message(raw_channel_message)
                                .map_err(|_| ())?,
                        ))
                    }
                },
            },
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum RawMessage {
    IdCodeMessage(RawIdCodeMessage),
    ChannelMessage(RawChannelMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawIdCodeMessage {
    pub id: i32,
    pub code: i32,
    #[serde(rename = "msg")]
    pub message: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawChannelMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "d")]
    pub data: RawChannelMessageData,
    #[serde(rename = "s")]
    pub symbol: Option<String>,
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum RawChannelMessageData {
    AccountDeals(RawAccountDealsData),
    AccountUpdate(RawAccountUpdateData),
    AccountOrders(RawAccountOrdersChannelMessageData),
    Event(RawEventChannelMessageData),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub(crate) enum RawEventChannelMessageData {
    Deals {
        deals: Vec<RawSpotDealData>,
        #[serde(rename = "e")]
        r#type: String,
    },
    Kline {
        k: RawKlineData,
        #[serde(rename = "e")]
        r#type: String,
    },
    OrdersUpdate {
        asks: Option<Vec<RawOrderData>>,
        bids: Option<Vec<RawOrderData>>,
        #[serde(rename = "r")]
        version: String,
        #[serde(rename = "e")]
        r#type: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_message_kline() {
        let json = r#"
            {"d":{"e":"spot@public.kline.v3.api","k":{"t":1695680400,"o":"26288.47","c":"26289.11","h":"26289.12","l":"26288.46","v":"1.579991","a":"41535.11","T":1695680460,"i":"Min1"}},"c":"spot@public.kline.v3.api@BTCUSDT@Min1","t":1695680458622,"s":"BTCUSDT"}
        "#;
        let deserializer = &mut serde_json::Deserializer::from_str(json);

        let result: Result<RawMessage, _> = serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn raw_channel_message_kline() {
        let json = r#"
            {"d":{"e":"spot@public.kline.v3.api","k":{"t":1695680400,"o":"26288.47","c":"26289.11","h":"26289.12","l":"26288.46","v":"1.579991","a":"41535.11","T":1695680460,"i":"Min1"}},"c":"spot@public.kline.v3.api@BTCUSDT@Min1","t":1695680458622,"s":"BTCUSDT"}
        "#;
        let deserializer = &mut serde_json::Deserializer::from_str(json);

        let result: Result<RawChannelMessage, _> = serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn raw_kline_data() {
        let json = r#"
            {"t":1695680400,"o":"26288.47","c":"26289.11","h":"26289.12","l":"26288.46","v":"1.579991","a":"41535.11","T":1695680460,"i":"Min1"}
        "#;
        let deserializer = &mut serde_json::Deserializer::from_str(json);

        let result: Result<RawKlineData, _> = serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn raw_event_data() {
        let json = r#"
            {"e":"spot@public.kline.v3.api","k":{"t":1695680400,"o":"26288.47","c":"26289.11","h":"26289.12","l":"26288.46","v":"1.579991","a":"41535.11","T":1695680460,"i":"Min1"}}
        "#;

        let deserializer = &mut serde_json::Deserializer::from_str(json);

        let result: Result<RawEventChannelMessageData, _> =
            serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn raw_orders_update_data() {
        let json = r#"
            { "d":{ "r":"3407459756", "e":"spot@public.increase.depth.v3.api", "asks":[{ "p":"20290.89", "v":"0.000000"}]}, "c": "spot@public.increase.depth.v3.api@BTCUSDT", "s":"BTCUSDT", "t":1661932660144}
        "#;

        let deserializer = &mut serde_json::Deserializer::from_str(json);

        let result: Result<RawChannelMessage, _> = serde_path_to_error::deserialize(deserializer);
        eprintln!("{:?}", result);
        assert!(result.is_ok());
    }
}
