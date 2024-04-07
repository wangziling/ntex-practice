use web_core::{error_prelude::*, features::RequestUtils, server_response_failed};

macro_rules! error_impl {
    ($ident:ident) => {
        impl From<$ident> for BoxedAppError {
            fn from(error: $ident) -> BoxedAppError {
                Box::new(error)
            }
        }

        impl WebResponseError for $ident {
            fn error_response(&self, req: &ntex::web::HttpRequest) -> ntex::http::Response {
                if req.wants_json() {
                    return server_response_failed!(message: Some(self.to_string())).into();
                }

                return self.response();
            }
        }
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
                if req.wants_json() {
                    return server_response_failed!(message: Some(self.to_string())).into();
                }

                return self.response();
            }
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum MiddlewareError {
    #[error("Json format required.")]
    RequireJsonFormat,
    #[error("Only for `ajax` remote call.")]
    ForAjaxReqOnly,
    #[error("App state missing.")]
    AppStateMissing,
}

#[derive(thiserror::Error, Debug)]
pub enum ExtensionError {
    #[error("Distribute cache missing.")]
    DistributeCacheMissing,
}

error_impl!(MiddlewareError, ntex::http::StatusCode::BAD_REQUEST);
error_impl!(ExtensionError);
