#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct WebsocketAuth {
    pub api_key: String,
    pub secret_key: String,
}
