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
    _api_key: String,
    secret_key: String,
}

impl MexcFuturesApiClientWithAuthentication {
    pub fn new(endpoint: MexcFuturesApiEndpoint, api_key: String, secret_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "ApiKey",
            api_key.parse().expect("Failed to parse api key"),
        );
        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client");
        Self {
            endpoint,
            reqwest_client,
            _api_key: api_key,
            secret_key,
        }
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
        let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");
        Self::new(MexcFuturesApiEndpoint::Base, api_key, secret_key)
    }
}
