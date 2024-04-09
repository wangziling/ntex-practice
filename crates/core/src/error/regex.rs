use super::{server_error_response, AppError, BoxedAppError};

#[derive(Debug)]
struct RegExpError {
    pub error: regex::Error,
}

impl AppError for RegExpError {
    fn response(&self) -> ntex::http::Response {
        error!(error = %self.error, "RegExp Error");

        server_error_response(format!("{}", self).into())
    }
}

impl std::fmt::Display for RegExpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegExp Error: {}", self.error)
    }
}

impl From<regex::Error> for BoxedAppError {
    fn from(error: regex::Error) -> Self {
        Box::new(RegExpError { error })
    }
}
