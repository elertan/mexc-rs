use crate::spot::wsv2::message::account_deals::{
    channel_message_to_account_deals_message, AccountDealsMessage, RawAccountDealsData,
};
use crate::spot::wsv2::message::account_orders::{
    channel_message_to_account_orders_message, AccountOrdersMessage,
    RawAccountOrdersChannelMessageData,
};
use crate::spot::wsv2::message::account_update::{
    channel_message_to_account_update_message, AccountUpdateMessage, RawAccountUpdateData,
};
use crate::spot::wsv2::message::deals::{
    channel_message_to_spot_deals_message, RawSpotDealData, SpotDealsMessage,
};
use crate::spot::wsv2::message::kline::{
    channel_message_to_spot_kline_message, RawKlineData, SpotKlineMessage,
};
use chrono::{DateTime, Utc};

pub mod account_deals;
pub mod account_orders;
pub mod account_update;
pub mod deals;
pub mod kline;

#[derive(Debug)]
pub enum Message {
    AccountDeals(AccountDealsMessage),
    AccountUpdate(AccountUpdateMessage),
    AccountOrders(AccountOrdersMessage),
    Deals(SpotDealsMessage),
    Kline(SpotKlineMessage),
}

impl TryFrom<&RawMessage> for Message {
    type Error = ();

    fn try_from(value: &RawMessage) -> Result<Self, Self::Error> {
        match value {
            RawMessage::IdCodeMessage(_) => Err(()),
            RawMessage::ChannelMessage(raw_channel_message) => match &raw_channel_message.data {
                RawChannelMessageData::AccountDeals(raw) => Ok(Message::AccountDeals(
                    channel_message_to_account_deals_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::AccountUpdate(raw) => Ok(Message::AccountUpdate(
                    channel_message_to_account_update_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::AccountOrders(raw) => Ok(Message::AccountOrders(
                    channel_message_to_account_orders_message(raw_channel_message)
                        .map_err(|_| ())?,
                )),
                RawChannelMessageData::Event(raw_event) => match &raw_event {
                    RawEventChannelMessageData::Deals(raw) => Ok(Message::Deals(
                        channel_message_to_spot_deals_message(raw_channel_message)
                            .map_err(|_| ())?,
                    )),
                    RawEventChannelMessageData::Kline(raw) => Ok(Message::Kline(
                        channel_message_to_spot_kline_message(raw_channel_message)
                            .map_err(|_| ())?,
                    )),
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
    #[serde(rename = "t")]
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
pub(crate) enum RawEventChannelMessageData {
    Deals(Vec<RawSpotDealData>),
    #[serde(rename = "k")]
    Kline(RawKlineData),
}
