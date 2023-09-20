pub mod subscription;

pub enum MexcWsEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcWsEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcWsEndpoint::Base => "wss://wbs.mexc.com/ws",
            MexcWsEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

pub struct MexcWsClient {
    endpoint: MexcWsEndpoint,
}

impl MexcWsClient {
    pub fn new(endpoint: MexcWsEndpoint) -> Self {
        Self { endpoint }
    }
}

impl Default for MexcWsClient {
    fn default() -> Self {
        Self::new(MexcWsEndpoint::Base)
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientMessagePayload<'a, T> {
    pub method: &'a str,
    pub params: T,
}
