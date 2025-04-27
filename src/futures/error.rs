// https://mxcdevelop.github.io/apidocs/contract_v1_en/#error-code-example

use crate::futures::response::ErrorApiResponse;
use crate::futures::GetAuthHeaderMapError;
use std::fmt::{Display, Formatter};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error response: {0:?}")]
    ErrorResponse(#[from] ErrorApiResponse),
    #[error("Get auth header map error: {0:?}")]
    GetAuthHeaderMapError(#[from] GetAuthHeaderMapError),
}

// 0 	Operate succeed
// 9999 	Public abnormal
// 500 	Internal error
// 501 	System busy
// 401 	Unauthorized
// 402 	Api_key expired
// 406 	Accessed IP is not in the whitelist
// 506 	Unknown source of request
// 510 	Excessive frequency of requests
// 511 	Endpoint inaccessible
// 513 	Invalid request(for open api serves time more or less than 10s)
// 600 	Parameter error
// 601 	Data decoding error
// 602 	Verify failed
// 603 	Repeated requests
// 701 	Account read permission is required
// 702 	Account modify permission is required
// 703 	Trade information read permission is required
// 704 	Transaction information modify permission is required
// 1000 	Account does not exist
// 1001 	Contract does not exist
// 1002 	Contract not activated
// 1003 	Error in risk limit level
// 1004 	Amount error
// 2001 	Wrong order direction
// 2002 	Wrong opening type
// 2003 	Overpriced to pay
// 2004 	Low-price for selling
// 2005 	Balance insufficient
// 2006 	Leverage ratio error
// 2007 	Order price error
// 2008 	The quantity is insufficient
// 2009 	Positions do not exist or have been closed
// 2011 	Order quantity error
// 2013 	Cancel orders over maximum limit
// 2014 	The quantity of batch order exceeds the limit
// 2015 	Price or quantity accuracy error
// 2016 	Trigger volume over the maximum
// 2018 	Exceeding the maximum available margin
// 2019 	There is an active open position
// 2021 	The single leverage is not consistent with the existing position leverage
// 2022 	Wrong position type
// 2023 	There are positions over the maximum leverage
// 2024 	There are orders with leverage over the maximum
// 2025 	The holding positions is over the maximum allowable positions
// 2026 	Modification of leverage is not supported for cross
// 2027 	There is only one cross or isolated in the same direction
// 2028 	The maximum order quantity is exceeded
// 2029 	Error order type
// 2030 	External order ID is too long (Max. 32 bits )
// 2031 	The allowable holding position exceed the current risk limit
// 2032 	Order price is less than long position force liquidate price
// 2033 	Order price is more than short position force liquidate price
// 2034 	The batch query quantity limit is exceeded
// 2035 	Unsupported market price tier
// 3001 	Trigger price type error
// 3002 	Trigger type error
// 3003 	Executive cycle error
// 3004 	Trigger price error
// 4001 	Unsupported currency
// 2036 	The orders more than the limit, please contact customer service
// 2037 	Frequent transactions, please try it later
// 2038 	The maximum allowable position quantity is exceeded, please contact customer service!
// 5001 	The take-price and the stop-loss price cannot be none at the same time
// 5002 	The Stop-Limit order does not exist or has closed
// 5003 	Take-profit and stop-loss price setting is wrong
// 5004 	The take-profit and stop-loss order volume is more than the holding positions can be liquidated
// 6001 	Trading forbidden
// 6002 	Open forbidden
// 6003 	Time range error
// 6004 	The trading pair and status should be fill in
// 6005 	The trading pair is not available

#[derive(Debug, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, strum_macros::IntoStaticStr)]
#[repr(i32)]
pub enum ErrorCode {
    OperationSucceed = 0,
    PublicAbnormal = 9999,
    InternalError = 500,
    SystemBusy = 501,
    Unauthorized = 401,
    ApiKeyExpired = 402,
    NotFound = 404,
    AccessedIpNotInWhitelist = 406,
    UnknownSourceOfRequest = 506,
    ExcessiveFrequencyOfRequests = 510,
    EndpointInaccessible = 511,
    InvalidRequest = 513,
    ParameterError = 600,
    DataDecodingError = 601,
    VerifyFailed = 602,
    RepeatedRequests = 603,
    AccountReadPermissionRequired = 701,
    AccountModifyPermissionRequired = 702,
    TradeInformationReadPermissionRequired = 703,
    TransactionInformationModifyPermissionRequired = 704,
    AccountDoesNotExist = 1000,
    ContractDoesNotExist = 1001,
    ContractNotActivated = 1002,
    ErrorInRiskLimitLevel = 1003,
    AmountError = 1004,
    WrongOrderDirection = 2001,
    WrongOpeningType = 2002,
    OverpricedToPay = 2003,
    LowPriceForSelling = 2004,
    BalanceInsufficient = 2005,
    LeverageRatioError = 2006,
    OrderPriceError = 2007,
    QuantityInsufficient = 2008,
    PositionsDoNotExistOrHaveBeenClosed = 2009,
    UnknownOrderSent = 2011,
    OrderQuantityError = 2012,
    CancelOrdersOverMaximumLimit = 2013,
    QuantityOfBatchOrderExceedsLimit = 2014,
    PriceOrQuantityAccuracyError = 2015,
    TriggerVolumeOverMaximum = 2016,
    ExceedingMaximumAvailableMargin = 2018,
    ThereIsActiveOpenPosition = 2019,
    SingleLeverageIsNotConsistentWithExistingPositionLeverage = 2021,
    WrongPositionType = 2022,
    PositionsOverMaximumLeverage = 2023,
    OrdersWithLeverageOverMaximum = 2024,
    HoldingPositionsOverMaximumAllowablePositions = 2025,
    ModificationOfLeverageIsNotSupportedForCross = 2026,
    ThereIsOnlyOneCrossOrIsolatedInTheSameDirection = 2027,
    MaximumOrderQuantityExceeded = 2028,
    ErrorOrderType = 2029,
    ExternalOrderIdIsTooLong = 2030,
    AllowableHoldingPositionExceedCurrentRiskLimit = 2031,
    OrderPriceIsLessThanLongPositionForceLiquidatePrice = 2032,
    OrderPriceIsMoreThanShortPositionForceLiquidatePrice = 2033,
    BatchQueryQuantityLimitExceeded = 2034,
    UnsupportedMarketPriceTier = 2035,
    TriggerPriceTypeError = 3001,
    TriggerTypeError = 3002,
    ExecutiveCycleError = 3003,
    TriggerPriceError = 3004,
    UnsupportedCurrency = 4001,
    OrdersMoreThanLimit = 2036,
    FrequentTransactions = 2037,
    MaximumAllowablePositionQuantityExceeded = 2038,
    TakePriceAndStopLossPriceCannotBeNoneAtTheSameTime = 5001,
    StopLimitOrderDoesNotExistOrHasClosed = 5002,
    TakeProfitAndStopLossPriceSettingIsWrong = 5003,
    TakeProfitAndStopLossOrderVolumeIsMoreThanHoldingPositionsCanBeLiquidated = 5004,
    TradingForbidden = 6001,
    OpenForbidden = 6002,
    TimeRangeError = 6003,
    TradingPairAndStatusShouldBeFillIn = 6004,
    TradingPairIsNotAvailable = 6005,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl std::error::Error for ErrorCode {}
