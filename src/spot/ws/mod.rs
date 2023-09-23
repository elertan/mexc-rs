

pub mod public;
pub mod private;

pub enum MexcSpotWsEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcSpotWsEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcSpotWsEndpoint::Base => "wss://wbs.mexc.com/ws",
            MexcSpotWsEndpoint::Custom(endpoint) => endpoint,
        }
    }
}
