#[cfg(feature = "enable-redis")]
pub mod redis;

#[derive(Clone, Debug)]
pub struct ErrorField(pub std::rc::Rc<BoxedAppError>);

pub type Result<Res, Err = anyhow::Error> = anyhow::Result<Res, Err>;

pub type BoxedAppError = Box<dyn AppError>;

pub type AppResult<Res, Err = BoxedAppError> = Result<Res, Err>;

pub trait AppError: 'static + std::fmt::Debug + std::fmt::Display + Send {
    fn response(&self) -> ntex::web::HttpResponse;

    fn cause(&self) -> Option<&dyn AppError> {
        None
    }

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
}

impl dyn AppError {
    pub fn is<T: std::any::Any>(&self) -> bool {
        self.type_id() == std::any::TypeId::of::<T>()
    }
}

impl AppError for BoxedAppError {
    fn response(&self) -> ntex::web::HttpResponse {
        (**self).response()
    }

    fn cause(&self) -> Option<&dyn AppError> {
        (**self).cause()
    }

    fn type_id(&self) -> std::any::TypeId {
        (**self).type_id()
    }
}

impl ntex::web::Responder for BoxedAppError {
    async fn respond_to(self, _req: &ntex::web::HttpRequest) -> ntex::http::Response {
        error!(error = %self, "Internal Server Error.");

        self.response()
    }
}

impl<E: std::error::Error + Send + 'static> AppError for E {
    fn response(&self) -> ntex::web::HttpResponse {
        error!(error = %self, "Internal Server Error.");

        server_error_response(self.to_string().into())
    }
}

// =============================================================================
// Error impls

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

impl From<sailfish::RenderError> for BoxedAppError {
    fn from(error: sailfish::RenderError) -> Self {
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

impl ErrorField {
    pub fn new(boxed_error: BoxedAppError) -> Self {
        Self(std::rc::Rc::new(boxed_error))
    }
}

impl ntex::web::WebResponseError for BoxedAppError {
    fn error_response(&self, _: &ntex::web::HttpRequest) -> ntex::http::Response {
        self.response()
    }
}

// =============================================================================
// Internal error

#[derive(Debug, Clone)]
pub struct InternalAppError {
    description: std::borrow::Cow<'static, str>,
}

impl std::fmt::Display for InternalAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.description)
    }
}

impl AppError for InternalAppError {
    fn response(&self) -> ntex::web::HttpResponse {
        error!(error = %self, "Internal server error.");

        let response = ntex::web::HttpResponse::new(ntex::http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body("Internal server error.".into());

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
    let error = InternalAppError { description };

    error.into()
}

pub fn server_error_response(description: std::borrow::Cow<'static, str>) -> ntex::web::HttpResponse {
    InternalAppError { description }.response()
}

pub fn anyhow_error(description: std::borrow::Cow<'static, str>) -> BoxedAppError {
    anyhow!(description).into()
}
