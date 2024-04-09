use super::{server_error_response, AppError, BoxedAppError};

#[derive(Debug)]
struct RedisError {
    pub error: fred::error::RedisError,
}

impl AppError for RedisError {
    fn response(&self) -> ntex::http::Response {
        error!(error = %self.error, "Redis Error");

        server_error_response(format!("{}", self).into())
    }
}

impl std::fmt::Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redis Error: {}", self.error)
    }
}

impl From<fred::error::RedisError> for BoxedAppError {
    fn from(error: fred::error::RedisError) -> Self {
        Box::new(RedisError { error })
    }
}
