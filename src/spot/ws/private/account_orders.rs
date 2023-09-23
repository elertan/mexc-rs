use std::str::FromStr;
use chrono::{DateTime, Utc};
use num_traits::FromPrimitive;
use rust_decimal::Decimal;
use crate::spot::v3::enums::{OrderSide, OrderStatus};
use crate::spot::ws::private::{AccountOrdersRawChannelMessageData, PrivateChannelMessage, PrivateChannelMessageData};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChannelMessageToAccountOrdersMessageError {
    #[error("Invalid channel message")]
    InvalidChannelMessage,
    #[error("Invalid order kind")]
    InvalidOrderKind,
    #[error("Invalid order status")]
    InvalidOrderStatus,
    #[error("Invalid stop limit direction")]
    InvalidStopLimitDirection,
    #[error("Invalid stop limit order state")]
    InvalidStopLimitOrderState
}

pub(crate) fn channel_message_to_account_orders_message(message: &PrivateChannelMessage) -> Result<AccountOrdersMessage, ChannelMessageToAccountOrdersMessageError> {
    let PrivateChannelMessageData::AccountOrders(account_orders_data) = &message.data else {
        return Err(ChannelMessageToAccountOrdersMessageError::InvalidChannelMessage);
    };
    let Some(asset) = &message.asset else {
        return Err(ChannelMessageToAccountOrdersMessageError::InvalidChannelMessage);
    };

    let message = match account_orders_data {
        AccountOrdersRawChannelMessageData::LimitOrMarket(limit_or_market) => {
            let msg = LimitOrMarketAccountOrdersMessage {
                symbol: asset.clone(),
                remain_amount: limit_or_market.A,
                create_time: limit_or_market.O,
                trade_type: if limit_or_market.S == 1 { OrderSide::Buy } else { OrderSide::Sell },
                remain_quantity: limit_or_market.V,
                amount: limit_or_market.a,
                client_order_id: limit_or_market.c.clone(),
                order_id: limit_or_market.i.clone(),
                is_maker: limit_or_market.m == 1,
                order_kind: OrderKind::from_u8(limit_or_market.o).ok_or(ChannelMessageToAccountOrdersMessageError::InvalidOrderKind)?,
                price: limit_or_market.p,
                status: match limit_or_market.s {
                    1 => OrderStatus::New,
                    2 => OrderStatus::Filled,
                    3 => OrderStatus::PartiallyFilled,
                    4 => OrderStatus::Canceled,
                    5 => OrderStatus::PartiallyCanceled,
                    _ => return Err(ChannelMessageToAccountOrdersMessageError::InvalidOrderStatus),
                },
                quantity: limit_or_market.v,
                average_price: limit_or_market.ap,
                cumulative_quantity: limit_or_market.cv,
                cumulative_amount: limit_or_market.ca,
            };
            AccountOrdersMessage::LimitOrMarket(msg)
        }
        AccountOrdersRawChannelMessageData::StopLimit(stop_limit) => {
            let msg = StopLimitAccountOrdersMessage {
                symbol: asset.clone(),
                commission_asset: stop_limit.N.clone(),
                create_time: stop_limit.O,
                trigger_price: stop_limit.P,
                trade_type: if stop_limit.S == 1 { OrderSide::Buy } else { OrderSide::Sell },
                direction: StopLimitDirection::from_u8(stop_limit.T).ok_or(ChannelMessageToAccountOrdersMessageError::InvalidStopLimitDirection)?,
                order_id: stop_limit.i.clone(),
                order_kind: OrderKind::from_u8(stop_limit.o).ok_or(ChannelMessageToAccountOrdersMessageError::InvalidOrderKind)?,
                price: stop_limit.p,
                state: StopLimitOrderState::from_u8(stop_limit.s).ok_or(ChannelMessageToAccountOrdersMessageError::InvalidStopLimitOrderState)?,
                quantity: stop_limit.v,
            };
            AccountOrdersMessage::StopLimit(msg)
        }
    };

    Ok(message)
}

#[derive(Debug)]
pub enum AccountOrdersMessage {
    LimitOrMarket(LimitOrMarketAccountOrdersMessage),
    StopLimit(StopLimitAccountOrdersMessage),
}

#[derive(Debug)]
pub struct LimitOrMarketAccountOrdersMessage {
    pub symbol: String,
    pub remain_amount: Decimal,
    pub create_time: DateTime<Utc>,
    pub trade_type: OrderSide,
    pub remain_quantity: Decimal,
    pub amount: Decimal,
    pub client_order_id: String,
    pub order_id: String,
    pub is_maker: bool,
    pub order_kind: OrderKind,
    pub price: Decimal,
    pub status: OrderStatus,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub cumulative_quantity: Decimal,
    pub cumulative_amount: Decimal,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum OrderKind {
    LimitOrder = 1,
    PostOnly = 2,
    ImmediateOrCancel = 3,
    FillOrKill = 4,
    MarketOrder = 5,
    StopLimit = 100,
}

#[derive(Debug)]
pub struct StopLimitAccountOrdersMessage {
    pub symbol: String,
    pub commission_asset: String,
    pub create_time: DateTime<Utc>,
    pub trigger_price: Decimal,
    pub trade_type: OrderSide,
    pub direction: StopLimitDirection,
    pub order_id: String,
    pub order_kind: OrderKind,
    pub price: Decimal,
    pub state: StopLimitOrderState,
    pub quantity: Decimal,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum StopLimitDirection {
    PriceHigherThanTriggerPrice = 0,
    PriceLowerThanTriggerPrice = 1,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum StopLimitOrderState {
    New = 0,
    Canceled = 1,
    Executed = 2,
    Failed = 3,
}

impl AsRef<str> for StopLimitOrderState {
    fn as_ref(&self) -> &str {
        match self {
            StopLimitOrderState::New => "NEW",
            StopLimitOrderState::Canceled => "CANCELED",
            StopLimitOrderState::Executed => "EXECUTED",
            StopLimitOrderState::Failed => "FAILED",
        }
    }
}

impl FromStr for StopLimitOrderState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(StopLimitOrderState::New),
            "CANCELED" => Ok(StopLimitOrderState::Canceled),
            "EXECUTED" => Ok(StopLimitOrderState::Executed),
            "FAILED" => Ok(StopLimitOrderState::Failed),
            _ => Err(()),
        }
    }
}

