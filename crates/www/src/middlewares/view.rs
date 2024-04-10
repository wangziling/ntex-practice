use web_core::middleware_prelude::*;
const HTTP_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/html; charset=utf-8");

pub fn prerequisites() -> ntex::web::middleware::DefaultHeaders {
    ntex::web::middleware::DefaultHeaders::new().header(ntex::http::header::CONTENT_TYPE, HTTP_CONTENT_TYPE)
}
