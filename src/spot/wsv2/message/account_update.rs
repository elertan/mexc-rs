use crate::spot::v3::enums::ChangedType;
use crate::spot::wsv2::message::{RawChannelMessage, RawChannelMessageData};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChannelMessageToAccountUpdateMessageError {
    #[error("Invalid channel message")]
    InvalidChannelMessage,
}

pub(crate) fn channel_message_to_account_update_message(
    message: &RawChannelMessage,
) -> Result<AccountUpdateMessage, ChannelMessageToAccountUpdateMessageError> {
    let RawChannelMessageData::AccountUpdate(account_update_data) = &message.data else {
        return Err(ChannelMessageToAccountUpdateMessageError::InvalidChannelMessage);
    };

    let message = AccountUpdateMessage {
        asset: account_update_data.a.clone(),
        change_time: account_update_data.c,
        free_balance: account_update_data.f,
        free_changed_amount: account_update_data.fd,
        frozen_amount: account_update_data.l,
        frozen_changed_amount: account_update_data.ld,
        changed_type: account_update_data.o,
        event_time: message.timestamp,
    };

    Ok(message)
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct RawAccountUpdateData {
    pub a: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub c: DateTime<Utc>,
    pub f: Decimal,
    pub fd: Decimal,
    pub l: Decimal,
    pub ld: Decimal,
    pub o: ChangedType,
}

#[derive(Debug)]
pub struct AccountUpdateMessage {
    pub asset: String,
    pub change_time: DateTime<Utc>,
    pub free_balance: Decimal,
    pub free_changed_amount: Decimal,
    pub frozen_amount: Decimal,
    pub frozen_changed_amount: Decimal,
    pub changed_type: ChangedType,
    pub event_time: DateTime<Utc>,
}
