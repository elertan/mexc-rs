use std::fmt::{Display, Formatter};
use reqwest::StatusCode;
use crate::spot::SignQueryError;

pub mod depth;
pub mod enums;
pub mod klines;
pub mod ping;
pub mod time;
pub mod default_symbols;
pub mod exchange_information;
pub mod trades;
pub mod order;
pub mod get_order;
pub mod cancel_order;
pub mod cancel_all_open_orders_on_a_symbol;
pub mod account_information;

pub type ApiResult<T> = Result<T, ApiError>;

// https://mxcdevelop.github.io/apidocs/spot_v3_en/#base-endpoint
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// HTTP 4XX return codes are used for malformed requests; the issue is on the sender's side.
    #[error("Malformed request")]
    MalformedRequest,

    /// HTTP 403 return code is used when the WAF Limit (Web Application Firewall) has been violated.
    #[error("Web application firewall (WAF) violated")]
    WebApplicationFirewallViolated,

    /// HTTP 429 return code is used when breaking a request rate limit.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// HTTP 5XX return codes are used for internal errors; the issue is on MEXC's side. It is important to NOT treat this as a failure operation; the execution status is UNKNOWN and could have been a success.
    #[error("Internal server error")]
    InternalServerError,

    #[error("Reqwest error: {0}")]
    ReqwestError(reqwest::Error),

    #[error("Serde url encoded error: {0}")]
    SerdeUrlencodedError(#[from] serde_urlencoded::ser::Error),

    #[error("Serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Unable to parse response
    #[error("Unable to parse response")]
    UnableToParseResponse,

    #[error("Error response: {0:?}")]
    ErrorResponse(#[from] ErrorResponse),

    #[error("Sign query error: {0}")]
    SignQueryError(#[from] SignQueryError),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        let status = match err.status() {
            None => {
                return Self::ReqwestError(err);
            }
            Some(status) => status,
        };

        match status {
            StatusCode::BAD_REQUEST => Self::MalformedRequest,
            StatusCode::FORBIDDEN => Self::WebApplicationFirewallViolated,
            StatusCode::TOO_MANY_REQUESTS => Self::RateLimitExceeded,
            StatusCode::INTERNAL_SERVER_ERROR => Self::InternalServerError,
            _ => Self::ReqwestError(err),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(T),
    Error(ErrorResponse),
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T, ErrorResponse> {
        match self {
            Self::Success(output) => Ok(output),
            Self::Error(err) => Err(err),
        }
    }

    pub fn into_api_result(self) -> ApiResult<T> {
        match self {
            Self::Success(output) => Ok(output),
            Self::Error(response) => Err(ApiError::ErrorResponse(response)),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub msg: String,
    pub _extend: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, strum_macros::IntoStaticStr)]
#[repr(i32)]
pub enum ErrorCode {
    UnknownOrderSent = -2011,
    OperationNotAllowed = 26,
    ApiKeyRequired = 400,
    NoAuthority = 401,
    AccessDenied = 403,
    TooManyRequests = 429,
    InternalError = 500,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    SignatureVerificationFailed = 602,
    UserDoesNotExist = 10001,
    BadSymbol = 10007,
    UserIdCannotBeNull = 10015,
    InvalidAccessKey = 10072,
    InvalidRequestTime = 10073,
    AmountCannotBeNull = 10095,
    AmountDecimalPlacesIsTooLong = 10096,
    AmountIsError = 10097,
    RiskControlSystemDetectedAbnormal = 10098,
    UserSubAccountDoesNotOpen = 10099,
    ThisCurrencyTransferIsNotSupported = 10100,
    InsufficientBalance = 10101,
    AmountCannotBeZeroOrNegative = 10102,
    ThisAccountTransferIsNotSupported = 10103,
    TransferOperationProcessing = 10200,
    TransferInFailed = 10201,
    TransferOutFailed = 10202,
    TransferIsDisabled = 10206,
    TransferIsForbidden = 10211,
    ThisWithdrawalAddressIsNotOnTheCommonlyUsedAddressListOrHasBeenInvalidated = 10212,
    NoAddressAvailablePleaseTryAgainLater = 10216,
    AssetFlowWritingFailedPleaseTryAgain = 10219,
    CurrencyCannotBeNull = 10222,
    CurrencyDoesNotExist = 10232,
    IntermediateAccountDoesNotConfiguredInRedisredis = 10259,
    DueToRiskControlWithdrawalIsUnavailablePleaseTryAgainLater = 10265,
    RemarkLengthIsTooLong = 10268,
    SubsystemIsNotSupported = 20001,
    InternalSystemErrorPleaseContactSupport = 20002,
    RecordDoesNotExist = 22222,
    SuspendedTransactionForTheSymbol = 30000,
    TheCurrentTransactionDirectionIsNotAllowedToPlaceAnOrder = 30001,
    TheMinimumTransactionVolumeCannotBeLessThan = 30002,
    TheMaximumTransactionVolumeCannotBeGreaterThan = 30003,
    InsufficientPosition = 30004,
    Oversold = 30005,
    NoValidTradePrice = 30010,
    InvalidSymbol = 30014,
    TradingDisabled = 30016,
    MarketOrderIsDisabled = 30018,
    ApiMarketOrderIsDisabled = 30019,
    NoPermissionForTheSymbol = 30020,
    NoExistOpponentOrder = 30025,
    InvalidOrderIds = 30026,
    TheCurrencyHasReachedTheMaximumPositionLimitTheBuyingIsSuspended = 30027,
    TheCurrencyTriggeredThePlatformRiskControlTheSellingIsSuspended = 30028,
    CannotExceedTheMaximumOrderLimit = 30029,
    CannotExceedTheMaximumPosition = 30032,
    CurrentOrderTypeCanNotPlaceOrder = 30041,
    ParamIsError = 33333,
    ParamCannotBeNull = 44444,
    YourAccountIsAbnormal = 60005,
    PairUserBanTradeApikey = 70011,
    ApiKeyFormatInvalid = 700001,
    SignatureForThisRequestIsNotValid = 700002,
    TimestampForThisRequestIsOutsideOfTheRecvWindow = 700003,
    ParamOrigClientOrderIdOrOrderIdMustBeSentButBothWereEmptyNull = 700004,
    RecvWindowMustLessThan60000 = 700005,
    IpNonWhiteList = 700006,
    NoPermissionToAccessTheEndpoint = 700007,
    IllegalCharactersFoundInParameter = 700008,
    // PairNotFound = 730001,
    // YourInputParamIsInvalid = 730002,
    RequestFailedPleaseContactTheCustomerService = 730000,
    // UserInformationError = 730001,
    PairNotFoundOrUserInformationError = 730001,
    // ParameterError = 730002,
    YourInputParamIsInvalidOrParameterError = 730002,
    UnsupportedOperationPleaseContactTheCustomerService = 730003,
    UnusualUserStatus = 730100,
    SubAccountNameCannotBeNull = 730600,
    SubAccountNameMustBeACombinationOf8To32LettersAndNumbers = 730601,
    SubAccountRemarksCannotBeNull = 730602,
    ApiKeyRemarksCannotBeNull = 730700,
    ApiKeyPermissionCannotBeNull = 730701,
    ApiKeyPermissionDoesNotExist = 730702,
    TheIpInformationIsIncorrectAndAMaximumOf10IPsAreAllowedToBeBoundOnly = 730703,
    TheBoundIpFormatIsIncorrectPleaseRefill = 730704,
    AtMost30GroupsOfApiKeysAreAllowedToBeCreatedOnly = 730705,
    ApiKeyInformationDoesNotExist = 730706,
    AccessKeyCannotBeNull = 730707,
    UserNameAlreadyExists = 730101,
    SubAccountDoesNotExist = 140001,
    SubAccountIsForbidden = 140002,
    OrderDoesNotExist = -2013,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl std::error::Error for ErrorCode {}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.code, self.msg)?;
        if let Some(extend) = &self._extend {
            write!(f, " (extend: {:?})", extend)?;
        }

        Ok(())
    }
}

impl std::error::Error for ErrorResponse {}
