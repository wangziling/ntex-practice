#[macro_use]
extern crate tracing;

#[macro_use]
extern crate anyhow;

#[macro_use]
mod macros;

pub mod constants;
pub mod error;
pub mod features;
pub mod response;
pub mod utils;
pub mod view_template;

pub mod prelude {
    pub use crate::error::Result;
}

pub mod error_prelude {
    pub use crate::error::{anyhow_error, internal_app_error, AppError, AppResult, BoxedAppError};
    pub use crate::prelude::*;

    pub use ntex::web::WebResponseError;
}

pub mod route_prelude {
    pub use crate::error::anyhow_error;
    pub use crate::features::{HttpRequestExt, RequestUtils, UriUtils};
    pub use crate::prelude::*;

    pub use ntex::web::{get, guard, post, resource, route, scope, to, Route, ServiceConfig};
}

pub mod middleware_prelude {
    pub use crate::error::{anyhow_error, BoxedAppError, ErrorField};
    pub use crate::features::{HttpRequestExt, RequestUtils, UriUtils};
    pub use crate::prelude::*;
    pub use crate::response::{map_view_render_result, HttpResponseExt, OriginalUrl, ResponseStatus, ServerResponse};
    pub use crate::server_redirect;
    pub use crate::server_response_failed;
    pub use crate::server_response_success;
    pub use crate::server_response_warning;
    pub use crate::utils::{query_to_string, remove_query, update_query, update_query_map};

    pub use ntex::http::header::HeaderValue;
    pub use ntex::http::{Method, StatusCode};
    pub use ntex::service::{Middleware, Service, ServiceCtx};
    pub use ntex::web::{Error, ErrorRenderer, WebRequest, WebResponse};
}

pub mod handler_prelude {
    pub use crate::error::{anyhow_error, AppResult, ErrorField};
    pub use crate::features::{HttpRequestExt, RequestUtils, UriUtils};
    pub use crate::prelude::*;
    pub use crate::response::{map_view_render_result, HttpResponseExt, OriginalUrl, ResponseStatus, ServerResponse};
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
