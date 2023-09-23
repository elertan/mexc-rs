use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::spot::v3::enums::ChangedType;
use crate::spot::ws::private::{PrivateChannelMessage, PrivateChannelMessageData};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChannelMessageToAccountUpdateMessageError {
    #[error("Invalid channel message")]
    InvalidChannelMessage,
}

pub(crate) fn channel_message_to_account_update_message(message: &PrivateChannelMessage) -> Result<AccountUpdateMessage, ChannelMessageToAccountUpdateMessageError> {
    let PrivateChannelMessageData::AccountUpdate(account_update_data) = &message.data else {
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
