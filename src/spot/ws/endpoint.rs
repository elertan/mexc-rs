use std::fmt;

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

impl fmt::Display for MexcWebsocketEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())  // Assure-toi que `as_ref()` retourne un type convertible en `Display`
    }
}