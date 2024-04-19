use web_core::error_prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum MiddlewareError {
    #[error("Json format required.")]
    RequireJsonFormat,
    #[error("Only for `ajax` remote call.")]
    ForAjaxReqOnly,
    #[error("App state missing.")]
    AppStateMissing,
}

app_error_impl!(MiddlewareError, ntex::http::StatusCode::BAD_REQUEST);
