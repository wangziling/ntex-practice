use serde::Serialize;

use crate::error::{AppResult, BoxedAppError};

pub fn redirect<U: AsRef<str>>(
    url: U,
    status_code: Option<ntex::http::StatusCode>,
) -> AppResult<ntex::web::HttpResponse> {
    let mut response = ntex::web::HttpResponse::new(status_code.unwrap_or_else(|| ntex::http::StatusCode::FOUND));

    response.headers_mut().insert(
        ntex::http::header::LOCATION,
        url.as_ref().parse::<ntex::http::header::HeaderValue>().map_err(Into::<BoxedAppError>::into)?,
    );

    Ok(response)
}

pub fn map_view_render_result(s: String) -> ntex::web::HttpResponse {
    ntex::web::HttpResponse::with_body(ntex::http::StatusCode::OK, s.into())
}

#[derive(Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    #[default]
    Success,
    Failed,
    Warning,
}

#[allow(unused)]
#[derive(Serialize)]
pub struct ServerResponse<D: Serialize, M: AsRef<str> + Serialize> {
    data: Option<D>,
    message: Option<M>,
    status: ResponseStatus,
    #[serde(skip)]
    status_code: ntex::http::StatusCode,
}

impl<D, M> ServerResponse<D, M>
where
    D: Serialize,
    M: AsRef<str> + Serialize,
{
    #[inline]
    pub fn failed(data: Option<D>, message: Option<M>, status_code: Option<ntex::http::StatusCode>) -> Self {
        Self {
            status: ResponseStatus::Failed,
            message,
            data,
            status_code: status_code.unwrap_or_else(|| ntex::http::StatusCode::OK),
        }
    }

    #[inline]
    pub fn success(data: Option<D>, message: Option<M>, status_code: Option<ntex::http::StatusCode>) -> Self {
        Self {
            status: ResponseStatus::Success,
            message,
            data,
            status_code: status_code.unwrap_or_else(|| ntex::http::StatusCode::OK),
        }
    }

    #[inline]
    pub fn warning(data: Option<D>, message: Option<M>, status_code: Option<ntex::http::StatusCode>) -> Self {
        Self {
            status: ResponseStatus::Warning,
            message,
            data,
            status_code: status_code.unwrap_or_else(|| ntex::http::StatusCode::OK),
        }
    }
}

impl<D, M> ntex::web::Responder for ServerResponse<D, M>
where
    D: Serialize,
    M: AsRef<str> + Serialize,
{
    async fn respond_to(self, _: &ntex::web::HttpRequest) -> ntex::http::Response {
        ntex::http::Response::from(self)
    }
}

impl<D, M> From<ServerResponse<D, M>> for ntex::http::Response
where
    D: Serialize,
    M: AsRef<str> + Serialize,
{
    fn from(value: ServerResponse<D, M>) -> Self {
        ntex::web::HttpResponseBuilder::new(value.status_code).json(&value)
    }
}

#[macro_export]
macro_rules! server_response_failed {
    () => {
        $crate::__server_response_impl!(failed)
    };

    // Named.
    (data: $data: expr) => {
        $crate::__server_response_impl!(failed, $data)
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, $message)
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, $data, Option::<String>::None, $status_code)
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, $message, $status_code)
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, $data, $message, $status_code)
    };

    // Ordered.
    ($($param:expr),+) => {
        $crate::__server_response_impl!(failed, $($param),+)
    };
}

#[macro_export]
macro_rules! server_response_success {
    () => {
        $crate::__server_response_impl!(success)
    };

    // Named.
    (data: $data: expr) => {
        $crate::__server_response_impl!(success, $data)
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, $message)
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, $data, Option::<String>::None, $status_code)
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, $message, $status_code)
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, $data, $message, $status_code)
    };

    // Ordered.
    ($($param:expr),+) => {
        $crate::__server_response_impl!(success, $($param),+)
    };
}

#[macro_export]
macro_rules! server_response_warning {
    () => {
        $crate::__server_response_impl!(warning)
    };

    // Named.
    (data: $data: expr) => {
        $crate::__server_response_impl!(warning, $data)
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, $message)
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, $data, Option::<String>::None, $status_code)
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, $message, $status_code)
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, $data, $message, $status_code)
    };

    // Ordered.
    ($($param:expr),+) => {
        $crate::__server_response_impl!(warning, $($param),+)
    };
}

#[macro_export]
macro_rules! server_redirect {
    ($url: expr) => {
        $crate::response::redirect($url, None)
    };

    ($url: expr, $status_code: expr) => {
        $crate::response::redirect($url, $status_code)
    };
}
