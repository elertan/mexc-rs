use hmac::digest::InvalidLength;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub mod v3;
#[cfg(feature = "ws")]
pub mod ws;

#[derive(Debug, Clone)]
pub enum MexcSpotApiEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcSpotApiEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcSpotApiEndpoint::Base => "https://api.mexc.com",
            MexcSpotApiEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

#[derive(Clone)]
pub struct MexcSpotApiClient {
    endpoint: MexcSpotApiEndpoint,
    reqwest_client: reqwest::Client,
}

impl MexcSpotApiClient {
    pub fn new(endpoint: MexcSpotApiEndpoint) -> Self {
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
    ) -> MexcSpotApiClientWithAuthentication {
        MexcSpotApiClientWithAuthentication::new(self.endpoint, api_key, secret_key)
    }
}

impl Default for MexcSpotApiClient {
    fn default() -> Self {
        Self::new(MexcSpotApiEndpoint::Base)
    }
}

#[derive(Clone)]
pub struct MexcSpotApiClientWithAuthentication {
    endpoint: MexcSpotApiEndpoint,
    reqwest_client: reqwest::Client,
    _api_key: String,
    secret_key: String,
}

impl MexcSpotApiClientWithAuthentication {
    pub fn new(endpoint: MexcSpotApiEndpoint, api_key: String, secret_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-MEXC-APIKEY",
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

    fn sign_query<T>(&self, query: T) -> Result<QueryWithSignature<T>, SignQueryError>
    where
        T: serde::Serialize,
    {
        let query_string = serde_urlencoded::to_string(&query)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())?;
        mac.update(query_string.as_bytes());
        let mac_result = mac.finalize();
        let mac_bytes = mac_result.into_bytes();
        let signature = hex::encode(mac_bytes);

        Ok(QueryWithSignature::new(query, signature))
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
        let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");
        Self::new(MexcSpotApiEndpoint::Base, api_key, secret_key)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryWithSignature<T> {
    #[serde(flatten)]
    pub query: T,
    pub signature: String,
}

impl<T> QueryWithSignature<T> {
    pub fn new(query: T, signature: String) -> Self {
        Self { query, signature }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SignQueryError {
    #[error("Serde url encoded error: {0}")]
    SerdeUrlencodedError(#[from] serde_urlencoded::ser::Error),

    #[error("Secret key invalid length")]
    SecretKeyInvalidLength(#[from] InvalidLength),
}
