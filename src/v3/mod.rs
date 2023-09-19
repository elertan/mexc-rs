use reqwest::StatusCode;

pub mod depth;
pub mod enums;
pub mod klines;
pub mod ping;
pub mod time;
pub mod default_symbols;
pub mod exchange_information;
pub mod trades;
pub mod order;

pub type ApiV3Result<T> = Result<T, ApiV3Error>;

// https://mxcdevelop.github.io/apidocs/spot_v3_en/#base-endpoint
#[derive(Debug, thiserror::Error)]
pub enum ApiV3Error {
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

    /// Unable to parse response
    #[error("Unable to parse response")]
    UnableToParseResponse,
}

impl From<reqwest::Error> for ApiV3Error {
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

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryWithSignature<'a, T> {
    #[serde(flatten)]
    pub query: T,
    pub signature: &'a str,
}

impl<'a, T> QueryWithSignature<'a, T> {
    pub fn new(query: T, signature: &'a str) -> Self {
        Self { query, signature }
    }
}
