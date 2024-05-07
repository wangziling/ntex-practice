use super::{server_error_response, AppError, BoxedAppError};
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
struct ViewTemplateError {
    error: sailfish::RenderError,
}

impl AppError for ViewTemplateError {
    fn response(&self) -> ntex::web::HttpResponse {
        server_error_response(format!("{}", self).into())
    }
}

impl std::fmt::Display for ViewTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "View Template Error: {}", self.error)
    }
}

impl From<sailfish::RenderError> for BoxedAppError {
    fn from(error: sailfish::RenderError) -> Self {
        Box::new(ViewTemplateError { error })
    }
}
