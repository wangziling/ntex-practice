use super::{server_error_response, AppError, BoxedAppError};

#[derive(Debug)]
struct RustlsError {
    pub error: anyhow::Error,
}

impl AppError for RustlsError {
    fn response(&self) -> ntex::http::Response {
        error!(error = %self.error, "Rustls Error");

        server_error_response(format!("{}", self).into())
    }
}

impl std::fmt::Display for RustlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rustls Error: {}", self.error)
    }
}

impl From<rustls::Error> for BoxedAppError {
    fn from(error: rustls::Error) -> Self {
        Box::new(RustlsError { error: anyhow::anyhow!(error.to_string()) })
    }
}

impl From<rustls_pemfile::Error> for BoxedAppError {
    fn from(error: rustls_pemfile::Error) -> Self {
        Box::new(RustlsError { error: anyhow::anyhow!(format!("{:?}", error)) })
    }
}
