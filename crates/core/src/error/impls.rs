use super::{internal_app_error, BoxedAppError};

impl From<serde_json::Error> for BoxedAppError {
    fn from(error: serde_json::Error) -> BoxedAppError {
        Box::new(error)
    }
}

impl From<std::io::Error> for BoxedAppError {
    fn from(error: std::io::Error) -> BoxedAppError {
        Box::new(error)
    }
}

impl From<std::convert::Infallible> for BoxedAppError {
    fn from(error: std::convert::Infallible) -> BoxedAppError {
        Box::new(error)
    }
}

impl From<anyhow::Error> for BoxedAppError {
    fn from(error: anyhow::Error) -> BoxedAppError {
        internal_app_error(error.to_string().into())
    }
}

impl From<ntex::http::header::InvalidHeaderValue> for BoxedAppError {
    fn from(_: ntex::http::header::InvalidHeaderValue) -> Self {
        internal_app_error("Invalid header value!".into())
    }
}

impl From<ntex::web::Error> for BoxedAppError {
    fn from(error: ntex::web::Error) -> Self {
        error.into()
    }
}

impl From<ntex::web::error::QueryPayloadError> for BoxedAppError {
    fn from(error: ntex::web::error::QueryPayloadError) -> Self {
        Box::new(error)
    }
}

impl From<serde_urlencoded::ser::Error> for BoxedAppError {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        Box::new(error)
    }
}

impl From<ntex::http::uri::InvalidUri> for BoxedAppError {
    fn from(error: ntex::http::uri::InvalidUri) -> Self {
        Box::new(error)
    }
}
