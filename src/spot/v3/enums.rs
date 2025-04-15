#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,

    Sell,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// Limit order
    Limit,

    /// Market order
    Market,

    /// Limit maker order
    LimitMaker,

    /// Immediate or cancel order
    ImmediateOrCancel,

    /// Fill or kill order
    FillOrKill,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Uncompleted
    New,

    /// Filled
    Filled,

    /// Partially filled
    PartiallyFilled,

    /// Canceled
    Canceled,

    /// Partially canceled
    PartiallyCanceled,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineInterval {
    /// 1 minute
    #[serde(rename = "1m")]
    OneMinute,

    /// 5 minutes
    #[serde(rename = "5m")]
    FiveMinutes,

    /// 15 minutes
    #[serde(rename = "15m")]
    FifteenMinutes,

    /// 30 minutes
    #[serde(rename = "30m")]
    ThirtyMinutes,

    /// 1 hour
    #[serde(rename = "60m")]
    OneHour,

    /// 4 hours
    #[serde(rename = "4h")]
    FourHours,

    /// 1 day
    #[serde(rename = "1d")]
    OneDay,

    /// 1 week
    #[serde(rename = "1W")]
    OneWeek,

    /// 1 month
    #[serde(rename = "1M")]
    OneMonth,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangedType {
    Withdraw,
    WithdrawFee,
    Deposit,
    DepositFee,
    Entrust,
    EntrustPlace,
    EntrustCancel,
    TradeFee,
    EntrustUnfrozen,
    Sugar,
    EtfIndex,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeType {
    Ask,
    Bid,
}
