use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use hmac::digest::InvalidLength;
use sha2::Sha256;

#[derive(Debug)]
pub struct SignRequestParams<'a, T> where T: serde::Serialize {
    pub time: DateTime<Utc>,
    pub api_key: &'a str,
    pub secret_key: &'a str,
    pub params: &'a T,
}

#[derive(Debug)]
pub struct SignRequestOutput {
    pub signature: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SignRequestError {
    #[error("Serde url encoded error: {0}")]
    SerdeUrlEncoded(#[from] serde_urlencoded::ser::Error),

    #[error("Secret key has invalid length for hmac sha 265")]
    SecretKeyInvalidLength(#[from] InvalidLength),
}

pub fn sign_request<T>(params: SignRequestParams<'_, T>) -> Result<SignRequestOutput, SignRequestError> where T: serde::Serialize {
    let query_string = serde_urlencoded::to_string(params.params)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(params.secret_key.as_bytes())?;

    let string_to_sign = format!("{}{}{}", params.api_key, params.time.timestamp_millis(), query_string);

    mac.update(string_to_sign.as_bytes());
    let mac_result = mac.finalize();
    let mac_bytes = mac_result.into_bytes();
    let signature = hex::encode(mac_bytes);

    Ok(SignRequestOutput {
        signature,
    })
}
