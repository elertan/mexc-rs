use chrono::Utc;
use crate::futures::auth::{SignRequestParams, SignRequestParamsKind};

pub mod error;
pub mod result;
pub mod response;
pub mod auth;
pub mod v1;

#[cfg(feature = "ws")]
pub mod ws;

pub enum MexcFuturesApiEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcFuturesApiEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcFuturesApiEndpoint::Base => "https://contract.mexc.com",
            MexcFuturesApiEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

pub struct MexcFuturesApiClient {
    endpoint: MexcFuturesApiEndpoint,
    reqwest_client: reqwest::Client,
}

impl MexcFuturesApiClient {
    pub fn new(endpoint: MexcFuturesApiEndpoint) -> Self {
        let reqwest_client = reqwest::Client::builder()
            .build()
            .expect("Failed to build reqwest client");
        Self {
            endpoint,
            reqwest_client,
        }
    }

    pub fn into_with_authentication(
        self,
        api_key: String,
        secret_key: String,
    ) -> MexcFuturesApiClientWithAuthentication {
        MexcFuturesApiClientWithAuthentication::new(self.endpoint, api_key, secret_key)
    }
}

impl Default for MexcFuturesApiClient {
    fn default() -> Self {
        Self::new(MexcFuturesApiEndpoint::Base)
    }
}

pub struct MexcFuturesApiClientWithAuthentication {
    endpoint: MexcFuturesApiEndpoint,
    reqwest_client: reqwest::Client,
    api_key: String,
    secret_key: String,
}

impl MexcFuturesApiClientWithAuthentication {
    pub fn new(endpoint: MexcFuturesApiEndpoint, api_key: String, secret_key: String) -> Self {
        let _headers = reqwest::header::HeaderMap::new();
        let reqwest_client = reqwest::Client::builder()
            .build()
            .expect("Failed to build reqwest client");
        Self {
            endpoint,
            reqwest_client,
            api_key,
            secret_key,
        }
    }

    fn get_auth_header_map<T>(&self, params: &T, kind: SignRequestParamsKind) -> Result<reqwest::header::HeaderMap, GetAuthHeaderMapError>
        where
            T: serde::Serialize,
    {
        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.insert(
            "ApiKey",
            self.api_key.parse().expect("Failed to parse api key"),
        );
        let now = Utc::now();
        header_map.insert(
            "Request-Time",
            now.timestamp_millis().to_string().parse().expect("Failed to parse request time"),
        );
        let sign_request_params = SignRequestParams {
            time: now,
            api_key: &self.api_key,
            secret_key: &self.secret_key,
            params,
            params_kind: kind,
        };
        let sign_request_output = auth::sign_request(sign_request_params)?;
        header_map.insert("Signature", sign_request_output.signature.parse().expect("Failed to parse signature"));
        Ok(header_map)
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
        let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");
        Self::new(MexcFuturesApiEndpoint::Base, api_key, secret_key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetAuthHeaderMapError {
    #[error("Sign request error: {0}")]
    SignRequestError(#[from] auth::SignRequestError),
}
