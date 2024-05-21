pub mod async_op_guard;
pub mod impls;
pub mod internal_error;
pub mod redis;
pub mod regex;
pub mod view_template;

#[cfg(feature = "tls-rustls")]
pub mod tls;

pub use internal_error::internal_app_error;
use internal_error::InternalAppError;

#[derive(Clone, Debug)]
pub struct ErrorField(std::rc::Rc<BoxedAppError>);

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

impl ntex::web::WebResponseError for BoxedAppError {
    fn error_response(&self, _: &ntex::web::HttpRequest) -> ntex::http::Response {
        self.response()
    }
}

impl<E: std::error::Error + Send + 'static> AppError for E {
    fn response(&self) -> ntex::web::HttpResponse {
        error!(error = %self, "Internal Server Error.");

        server_error_response(self.to_string().into())
    }
}

impl ErrorField {
    pub fn new(boxed_error: BoxedAppError) -> Self {
        Self(std::rc::Rc::new(boxed_error))
    }
}

impl std::ops::Deref for ErrorField {
    type Target = std::rc::Rc<BoxedAppError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn server_error_response(description: std::borrow::Cow<'static, str>) -> ntex::web::HttpResponse {
    InternalAppError { description }.response()
}

pub fn anyhow_error(description: std::borrow::Cow<'static, str>) -> BoxedAppError {
    anyhow!(description).into()
}

pub trait AppErrorExt: ToString {
    fn as_app_error(&self) -> BoxedAppError {
        anyhow_error(self.to_string().into())
    }
}

#[macro_export]
macro_rules! app_error_impl {
    ($ident:ident) => {
        impl From<$ident> for BoxedAppError {
            fn from(error: $ident) -> BoxedAppError {
                Box::new(error)
            }
        }

        impl WebResponseError for $ident {
            fn error_response(&self, req: &ntex::web::HttpRequest) -> ntex::http::Response {
                use $crate::features::RequestUtils;

                if req.wants_json() {
                    return $crate::server_response_failed!(message: self.to_string()).into();
                }

                return self.response();
            }
        }

        impl $crate::error::AppErrorExt for $ident {}
    };

    ($ident:ident, $status_code:expr) => {
        impl From<$ident> for BoxedAppError {
            fn from(error: $ident) -> BoxedAppError {
                Box::new(error)
            }
        }

        impl WebResponseError for $ident {
            fn status_code(&self) -> ntex::http::StatusCode {
                $status_code
            }

            fn error_response(&self, req: &ntex::web::HttpRequest) -> ntex::http::Response {
                use $crate::features::RequestUtils;

                if req.wants_json() {
                    return $crate::server_response_failed!(message: self.to_string()).into();
                }

                return self.response();
            }
        }

        impl $crate::error::AppErrorExt for $ident {}
    };
}
