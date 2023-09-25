#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Message {
    IdCodeMessage(IdCodeMessage),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdCodeMessage {
    pub id: i32,
    pub code: i32,
    #[serde(rename = "msg")]
    pub message: String,
}
