use web_core::{error_prelude::*, features::RequestUtils, server_response_failed};

#[derive(thiserror::Error, Debug)]
pub enum MiddlewareError {
    #[error("Json format required.")]
    RequireJsonFormat,
    #[error("Only for `ajax` remote call.")]
    ForAjaxReqOnly,
    #[error("App state missing.")]
    AppStateMissing,
}

impl From<MiddlewareError> for BoxedAppError {
    fn from(error: MiddlewareError) -> BoxedAppError {
        Box::new(error)
    }
}

impl WebResponseError for MiddlewareError {
    fn status_code(&self) -> ntex::http::StatusCode {
        ntex::http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self, req: &ntex::web::HttpRequest) -> ntex::http::Response {
        if req.wants_json() {
            return server_response_failed!(message: Some(self.to_string())).into();
        }

        return self.response();
    }
}
