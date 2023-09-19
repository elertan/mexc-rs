pub mod v3;

pub enum MexcApiEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcApiEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcApiEndpoint::Base => "https://api.mexc.com",
            MexcApiEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

pub struct MexcApiClient {
    endpoint: MexcApiEndpoint,
    reqwest_client: reqwest::Client,
}

impl MexcApiClient {
    pub fn new(endpoint: MexcApiEndpoint) -> Self {
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
    ) -> MexcApiClientWithAuthentication {
        MexcApiClientWithAuthentication::new(self.endpoint, api_key, secret_key)
    }
}

impl Default for MexcApiClient {
    fn default() -> Self {
        Self::new(MexcApiEndpoint::Base)
    }
}

pub struct MexcApiClientWithAuthentication {
    endpoint: MexcApiEndpoint,
    reqwest_client: reqwest::Client,
    api_key: String,
    secret_key: String,
}

impl MexcApiClientWithAuthentication {
    pub fn new(endpoint: MexcApiEndpoint, api_key: String, secret_key: String) -> Self {
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
            api_key,
            secret_key,
        }
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
        let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");
        Self::new(MexcApiEndpoint::Base, api_key, secret_key)
    }
}
