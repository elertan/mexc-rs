#[derive(Debug)]
pub enum MexcWebsocketEndpoint {
    Base,
    Custom(String),
}

impl AsRef<str> for MexcWebsocketEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MexcWebsocketEndpoint::Base => "wss://wbs.mexc.com/ws",
            MexcWebsocketEndpoint::Custom(endpoint) => endpoint,
        }
    }
}

impl ToString for MexcWebsocketEndpoint {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
