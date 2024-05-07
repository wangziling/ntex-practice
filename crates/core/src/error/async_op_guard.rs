use super::{server_error_response, AppError, BoxedAppError};
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
struct AsyncOpGuardError {
    error: rslock::LockError,
}

impl AppError for AsyncOpGuardError {
    fn response(&self) -> ntex::http::Response {
        error!(error = %format!("{:?}", self.error), "AsyncOpGuard Error");

        server_error_response(format!("{:?}", self.error).into())
    }
}

impl std::fmt::Display for AsyncOpGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncOpGuard Error: {:?}", self.error)
    }
}

impl From<rslock::LockError> for BoxedAppError {
    fn from(error: rslock::LockError) -> Self {
        Box::new(AsyncOpGuardError { error })
    }
}
