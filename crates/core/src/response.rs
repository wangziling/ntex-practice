use crate::error::{AppResult, BoxedAppError};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub struct OriginalUrl(String);

impl OriginalUrl {
    pub fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl std::ops::Deref for OriginalUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn redirect<U: AsRef<str>, S: TryInto<ntex::http::StatusCode>>(
    url: U,
    prev_url: Option<String>,
    status_code: Option<S>,
) -> AppResult<ntex::web::HttpResponse> {
    let mut response = ntex::web::HttpResponse::new(
        status_code.and_then(crate::utils::parse_into_status_code).unwrap_or_else(|| ntex::http::StatusCode::FOUND),
    );

    response.headers_mut().insert(
        ntex::http::header::LOCATION,
        url.as_ref().parse::<ntex::http::header::HeaderValue>().map_err(Into::<BoxedAppError>::into)?,
    );

    if prev_url.is_some() {
        response.extensions_mut().insert(OriginalUrl(prev_url.unwrap()));
    }

    Ok(response)
}

pub fn map_view_render_result(s: String) -> ntex::web::HttpResponse {
    ntex::web::HttpResponse::with_body(ntex::http::StatusCode::OK, s.into())
}

pub trait HttpResponseExt<Err> {
    fn into_web_response(self, req: ntex::web::WebRequest<Err>) -> ntex::web::WebResponse;
}

impl<Err: ntex::web::ErrorRenderer> HttpResponseExt<Err> for ntex::web::HttpResponse {
    fn into_web_response(self, req: ntex::web::WebRequest<Err>) -> ntex::web::WebResponse {
        ntex::web::WebResponse::new(self, req.into_parts().0)
    }
}

#[derive(Serialize, Default, Deserialize, PartialEq, Eq, Debug, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    #[default]
    Success,
    Failed,
    Warning,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, ToSchema)]
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
    pub fn failed<S>(data: Option<D>, message: Option<M>, status_code: Option<S>) -> Self
    where
        S: TryInto<ntex::http::StatusCode>,
    {
        Self {
            status: ResponseStatus::Failed,
            message,
            data,
            status_code: status_code
                .and_then(crate::utils::parse_into_status_code)
                .unwrap_or_else(|| ntex::http::StatusCode::OK),
        }
    }

    #[inline]
    pub fn success<S>(data: Option<D>, message: Option<M>, status_code: Option<S>) -> Self
    where
        S: TryInto<ntex::http::StatusCode>,
    {
        Self {
            status: ResponseStatus::Success,
            message,
            data,
            status_code: status_code
                .and_then(crate::utils::parse_into_status_code)
                .unwrap_or_else(|| ntex::http::StatusCode::OK),
        }
    }

    #[inline]
    pub fn warning<S>(data: Option<D>, message: Option<M>, status_code: Option<S>) -> Self
    where
        S: TryInto<ntex::http::StatusCode>,
    {
        Self {
            status: ResponseStatus::Warning,
            message,
            data,
            status_code: status_code
                .and_then(crate::utils::parse_into_status_code)
                .unwrap_or_else(|| ntex::http::StatusCode::OK),
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
        $crate::__server_response_impl!(failed, Some($data))
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, Some($message))
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Some($data), Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, message: $message: expr) => {
        $crate::__server_response_impl!(failed, Some($data), Some($message))
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, Some($message), Some($status_code))
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Some($data), Some($message), Some($status_code))
    };


    // Optional.
    (optional_data: $data: expr) => {
        $crate::__server_response_impl!(failed, $data)
    };
    (optional_message: $message: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, $message)
    };
    (optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, $data, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_message: $optional_message: expr) => {
        $crate::__server_response_impl!(failed, $data, $optional_message)
    };
    (optional_message: $message: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(failed, Option::<String>::None, $message, $status_code)
    };
    (optional_data: $data: expr, optional_message: $message: expr, optional_status_code: $status_code: expr) => {
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
        $crate::__server_response_impl!(success, Some($data))
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, Some($message))
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Some($data), Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, message: $message: expr) => {
        $crate::__server_response_impl!(success, Some($data), Some($message))
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, Some($message), Some($status_code))
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Some($data), Some($message), Some($status_code))
    };

    // Optional.
    (optional_data: $data: expr) => {
        $crate::__server_response_impl!(success, $data)
    };
    (optional_message: $message: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, $message)
    };
    (optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, $data, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_message: $optional_message: expr) => {
        $crate::__server_response_impl!(success, $data, $optional_message)
    };
    (optional_message: $message: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(success, Option::<String>::None, $message, $status_code)
    };
    (optional_data: $data: expr, optional_message: $message: expr, optional_status_code: $status_code: expr) => {
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
        $crate::__server_response_impl!(warning, Some($data))
    };
    (message: $message: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, Some($message))
    };
    (status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Some($data), Option::<String>::None, Some($status_code))
    };
    (data: $data: expr, message: $message: expr) => {
        $crate::__server_response_impl!(warning, Some($data), Some($message))
    };
    (message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, Some($message), Some($status_code))
    };
    (data: $data: expr, message: $message: expr, status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Some($data), Some($message), Some($status_code))
    };

    // Optional.
    (optional_data: $data: expr) => {
        $crate::__server_response_impl!(warning, $data)
    };
    (optional_message: $message: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, $message)
    };
    (optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, $data, Option::<String>::None, $status_code)
    };
    (optional_data: $data: expr, optional_message: $optional_message: expr) => {
        $crate::__server_response_impl!(warning, $data, $optional_message)
    };
    (optional_message: $message: expr, optional_status_code: $status_code: expr) => {
        $crate::__server_response_impl!(warning, Option::<String>::None, $message, $status_code)
    };
    (optional_data: $data: expr, optional_message: $message: expr, optional_status_code: $status_code: expr) => {
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
        $crate::response::redirect($url, None, Option::<u16>::None)
    };
    ($url: expr, $prev_url: expr) => {
        $crate::response::redirect($url, $prev_url, Option::<u16>::None)
    };
    ($url: expr, $prev_url: expr, $status_code: expr) => {
        $crate::response::redirect($url, $prev_url, $status_code)
    };
    ($url: expr, prev_url:$prev_url: expr) => {
        $crate::server_redirect!($url, Some($prev_url), Option::<u16>::None)
    };
    ($url: expr, status_code:$status_code: expr) => {
        $crate::server_redirect!($url, None, Some($status_code))
    };
    ($url: expr, prev_url:$prev_url: expr, status_code:$status_code: expr) => {
        $crate::server_redirect!($url, Some($prev_url), Some($status_code))
    };

    // Optional.
    ($url: expr, prev_url:$prev_url: expr, optional_status_code:$status_code: expr) => {
        $crate::server_redirect!($url, Some($prev_url), $status_code)
    };
    ($url: expr, optional_status_code:$status_code: expr) => {
        $crate::server_redirect!($url, None, $status_code)
    };
    ($url: expr, optional_prev_url:$prev_url: expr) => {
        $crate::server_redirect!($url, $prev_url, Option::<u16>::None)
    };
    ($url: expr, optional_prev_url:$prev_url: expr, optional_status_code:$status_code: expr) => {
        $crate::server_redirect!($url, $prev_url, $status_code)
    };
}

#[cfg(test)]
mod macro_tests {
    #![allow(unused_doc_comments)]

    use crate::*;

    macro_rules! __server_response_test_impl {
        ($type: ident) => {
            paste::item! {
                #[test]
                fn [<server_response_ $type>] () {
                    let _ = [<server_response_ $type>]!();
                    let _ = [<server_response_ $type>]!(/** data */Option::<String>::None);
                    let _ = [<server_response_ $type>]!(/** data */Some(""));
                    let _ = [<server_response_ $type>]!(/** data */Some(""), /** message */Option::<String>::None);
                    let _ = [<server_response_ $type>]!(/** data */Some(""), /** message */Some(""));
                    let _ = [<server_response_ $type>]!(/** data */Some(""), /** message */Option::<String>::None, /** status_code */Option::<u16>::None);
                    let _ = [<server_response_ $type>]!(/** data */Some(""), /** message */Some(""), /** status_code */Some(404));

                    // Named.
                    let _ = [<server_response_ $type>]!(data: "");
                    let _ = [<server_response_ $type>]!(data: "", message: "");
                    let _ = [<server_response_ $type>]!(data: "", message: "", status_code: 404);
                    let _ = [<server_response_ $type>]!(data: "", message: "", status_code: ntex::http::StatusCode::NOT_FOUND);
                    let _ = [<server_response_ $type>]!(message: "");
                    let _ = [<server_response_ $type>]!(message: "", status_code: 404);
                    let _ = [<server_response_ $type>]!(message: "", status_code: ntex::http::StatusCode::NOT_FOUND);
                    let _ = [<server_response_ $type>]!(status_code: 404);
                    let _ = [<server_response_ $type>]!(status_code: ntex::http::StatusCode::NOT_FOUND);

                    // Optional.
                    let _ = [<server_response_ $type>]!(optional_data: Option::<String>::None);
                    let _ = [<server_response_ $type>]!(optional_data: Some(""));
                    let _ =
                        [<server_response_ $type>]!(optional_data: Option::<String>::None, optional_message: Option::<String>::None);
                    let _ = [<server_response_ $type>]!(optional_data: Some(""), optional_message: Some(""));
                    let _ = [<server_response_ $type>]!(optional_data: Option::<String>::None, optional_message: Option::<String>::None, optional_status_code: Option::<u16>::None);
                    let _ = [<server_response_ $type>]!(optional_data: Some(""), optional_message: Some(""), optional_status_code: Some(404));
                }
            }
        };
    }

    #[test]
    fn server_redirect() {
        let _ = server_redirect!(/** url */"");
        let _ = server_redirect!(/** url */"", /** prev_url */Some("".to_string()));
        let _ = server_redirect!(/** url */"", /** prev_url */Some("".to_string()), /** status_code */Some(404));

        // Named.
        let _ = server_redirect!(/** url */"", prev_url: "".to_string());
        let _ = server_redirect!(/** url */"", status_code: 404);
        let _ = server_redirect!(/** url */"", status_code: ntex::http::StatusCode::NOT_FOUND);
        let _ = server_redirect!(/** url */"", prev_url: "".to_string(), status_code: 404);
        let _ =
            server_redirect!(/** url */"", prev_url: "".to_string(), status_code: ntex::http::StatusCode::NOT_FOUND);

        // Optional.
        let _ = server_redirect!(/** url */"", optional_prev_url: None);
        let _ = server_redirect!(/** url */"", optional_prev_url: Some("".to_string()));
        let _ = server_redirect!(/** url */"", optional_status_code: Option::<u16>::None);
        let _ = server_redirect!(/** url */"", optional_status_code: Some(404));
        let _ = server_redirect!(/** url */"", optional_status_code: Some(ntex::http::StatusCode::NOT_FOUND));
        let _ = server_redirect!(/** url */"", optional_prev_url: None, optional_status_code: Option::<u16>::None);
        let _ = server_redirect!(/** url */"", optional_prev_url: Some("".to_string()), optional_status_code: Option::<u16>::None);
        let _ = server_redirect!(/** url */"", optional_prev_url: None, optional_status_code: Some(404));
        let _ =
            server_redirect!(/** url */"", optional_prev_url: Some("".to_string()), optional_status_code: Some(404));
        let _ = server_redirect!(/** url */"", optional_prev_url: None, optional_status_code: Some(ntex::http::StatusCode::NOT_FOUND));
        let _ = server_redirect!(/** url */"", optional_prev_url: Some("".to_string()), optional_status_code: Some(ntex::http::StatusCode::NOT_FOUND));
    }

    __server_response_test_impl!(success);
    __server_response_test_impl!(failed);
    __server_response_test_impl!(warning);
}
