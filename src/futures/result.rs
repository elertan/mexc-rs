use crate::futures::error::ApiError;

pub type ApiResult<T> = Result<T, ApiError>;
