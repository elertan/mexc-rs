#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct WebsocketAuth {
    pub api_key: String,
    pub secret_key: String,
}

impl WebsocketAuth {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }
}
