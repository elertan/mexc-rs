use crate::futures::error::{ApiError, ErrorCode};
use crate::futures::result::ApiResult;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(SuccessApiResponse<T>),
    Error(ErrorApiResponse),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessApiResponse<T> {
    pub data: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorApiResponse {
    pub code: ErrorCode,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T, ErrorApiResponse> {
        match self {
            Self::Success(output) => Ok(output.data),
            Self::Error(err) => Err(err),
        }
    }

    pub fn into_api_result(self) -> ApiResult<T> {
        match self {
            Self::Success(output) => Ok(output.data),
            Self::Error(response) => Err(ApiError::ErrorResponse(response)),
        }
    }
}

impl Error for ErrorApiResponse {}

impl Display for ErrorApiResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error response: code: {}, msg: {}",
            self.code, self.message
        )
    }
}
