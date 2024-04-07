#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

pub mod error;
pub mod features;
pub mod response;
pub mod view_template;

pub mod prelude {
    pub use crate::error::internal_app_error;
    pub use crate::error::Result;
}

pub mod error_prelude {
    pub use crate::error::AppError;
    pub use crate::error::AppResult;
    pub use crate::error::BoxedAppError;
    pub use crate::prelude::*;

    pub use ntex::web::WebResponseError;
}

pub mod route_prelude {
    pub use crate::prelude::*;
    pub use ntex::http::RequestHead;
    pub use ntex::web::{get, guard, post, resource, route, scope, to, ServiceConfig};
}

pub mod middleware_prelude {
    pub use crate::features::HttpRequestExt;
    pub use crate::features::RequestUtils;
    pub use crate::prelude::*;
    pub use crate::response::map_view_render_result;
    pub use crate::response::ResponseStatus;
    pub use crate::response::ServerResponse;
    pub use crate::server_redirect;
    pub use crate::server_response_failed;
    pub use crate::server_response_success;
    pub use crate::server_response_warning;

    pub use ntex::service::{Middleware, Service, ServiceCtx};
    pub use ntex::web::{Error, ErrorRenderer, WebRequest, WebResponse};
}

pub mod handler_prelude {
    pub use crate::error::AppResult;
    pub use crate::features::HttpRequestExt;
    pub use crate::features::RequestUtils;
    pub use crate::prelude::*;
    pub use crate::response::map_view_render_result;
    pub use crate::response::ResponseStatus;
    pub use crate::response::ServerResponse;
    pub use crate::server_redirect;
    pub use crate::server_response_failed;
    pub use crate::server_response_success;
    pub use crate::server_response_warning;

    pub use sailfish::TemplateOnce;
    pub use serde::{Deserialize, Serialize};
    pub use web_proc_macros::web_view_template;

    pub use ntex::http::header::HeaderValue;
    pub use ntex::http::StatusCode;
    pub use ntex::web::types::{Form, Json, Path, Payload, State};
    pub use ntex::web::{get, post, HttpRequest, HttpResponse, Responder};
}
