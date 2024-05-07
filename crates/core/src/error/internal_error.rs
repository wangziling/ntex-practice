use super::{AppError, BoxedAppError, ErrorField};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
pub struct InternalAppError {
    pub description: std::borrow::Cow<'static, str>,
}

impl std::fmt::Display for InternalAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl AppError for InternalAppError {
    fn response(&self) -> ntex::web::HttpResponse {
        error!(error = %self, "Internal server error.");

        let response = ntex::web::HttpResponse::new(ntex::http::StatusCode::INTERNAL_SERVER_ERROR);

        response.extensions_mut().insert(ErrorField::new(self.clone().into()));

        response
    }
}

impl From<InternalAppError> for BoxedAppError {
    fn from(error: InternalAppError) -> Self {
        Box::new(error)
    }
}

pub fn internal_app_error(description: std::borrow::Cow<'static, str>) -> BoxedAppError {
    InternalAppError { description }.into()
}
