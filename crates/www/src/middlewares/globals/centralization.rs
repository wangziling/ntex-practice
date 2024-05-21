use crate::constants::{INTERNAL_SERVER_ERROR_REQ_PATH, NOT_FOUND_REQ_PATH};
use web_core::middleware_prelude::*;

const NOT_FOUND_MESSAGE: &str = "Requested resource not found.";
const INTERNAL_SERVER_ERROR_MESSAGE: &str = "Internal Server Error.";
const PREV_URL_SEARCH_QUERY_KEY: &str = "prev";

pub struct Centralization;

impl<S> Middleware<S> for Centralization {
    type Service = CentralizationInner<S>;

    fn create(&self, service: S) -> Self::Service {
        CentralizationInner { service }
    }
}

pub struct CentralizationInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for CentralizationInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let mut res = ctx.call(&self.service, req).await?;

        let req = res.request();
        match res.status() {
            StatusCode::INTERNAL_SERVER_ERROR if !req.path().eq(INTERNAL_SERVER_ERROR_REQ_PATH) => {
                if req.wants_json() {
                    *res.response_mut() =
                        server_response_failed!(message: INTERNAL_SERVER_ERROR_MESSAGE, status_code: 500).into();

                    return Ok(res);
                }

                if !req.derived_from_ajax() && req.method() == Method::GET {
                    // UNWRAP: Operation must be successful.
                    let mut uri = INTERNAL_SERVER_ERROR_REQ_PATH.parse::<ntex::http::Uri>().unwrap();

                    if let Some(path_query) = req.uri().path_and_query() {
                        let query_map = uri
                            .update_query(PREV_URL_SEARCH_QUERY_KEY.to_string(), Some(path_query.to_string()))
                            .map_err(Into::<BoxedAppError>::into)?;

                        match query_to_string(query_map) {
                            Ok(query_map_string) if !query_map_string.is_empty() => {
                                *res.response_mut() = server_redirect!(uri.path().to_string() + "?" + &query_map_string, prev_url: req.uri().to_string())?;

                                return Ok(res);
                            }
                            _ => {}
                        }
                    }

                    *res.response_mut() = server_redirect!(uri.path(), prev_url: req.uri().to_string())?;

                    return Ok(res);
                }
            }
            StatusCode::NOT_FOUND if !req.path().eq(NOT_FOUND_REQ_PATH) => {
                if req.wants_json() {
                    *res.response_mut() = server_response_failed!(message: NOT_FOUND_MESSAGE, status_code: 404).into();

                    return Ok(res);
                }

                if !req.derived_from_ajax() && req.method() == Method::GET {
                    // UNWRAP: Operation must be successful.
                    let mut uri = NOT_FOUND_REQ_PATH.parse::<ntex::http::Uri>().unwrap();

                    if let Some(path_query) = req.uri().path_and_query() {
                        let query_map = uri
                            .update_query(PREV_URL_SEARCH_QUERY_KEY.to_string(), Some(path_query.to_string()))
                            .map_err(Into::<BoxedAppError>::into)?;

                        match query_to_string(query_map) {
                            Ok(query_map_string) if !query_map_string.is_empty() => {
                                *res.response_mut() = server_redirect!(uri.path().to_string() + "?" + &query_map_string, prev_url: req.uri().to_string())?;

                                return Ok(res);
                            }
                            _ => {}
                        }
                    }

                    *res.response_mut() = server_redirect!(uri.path(), prev_url: req.uri().to_string())?;

                    return Ok(res);
                }
            }
            _ => {}
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ntex::http::{header, header::HeaderName, header::HeaderValue, StatusCode, Uri};
    use ntex::service::{IntoService, Middleware, Pipeline};
    use ntex::util::{lazy, Bytes};
    use ntex::web::test::{init_service, read_body, TestRequest};
    use ntex::web::{resource, App, DefaultError, Error, HttpResponse};
    use once_cell::sync::Lazy;

    use super::{
        server_response_failed, Centralization, Method, OriginalUrl, ServerResponse, INTERNAL_SERVER_ERROR_MESSAGE,
        INTERNAL_SERVER_ERROR_REQ_PATH, NOT_FOUND_MESSAGE, NOT_FOUND_REQ_PATH, PREV_URL_SEARCH_QUERY_KEY,
    };
    use web_core::constants::{JSON_HEADER_VALUE, REQUESTED_WITH_AJAX_HEADER_VALUE, REQUESTED_WITH_HEADER_NAME};

    macro_rules! init_service {
        () => {
            init_service(
                App::new()
                    .wrap(Centralization)
                    .service(resource("/test").to(|| async { HttpResponse::Ok() }))
                    .service(
                        resource("/test-500").to(|| async { HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR) }),
                    )
                    .service(resource(NOT_FOUND_REQ_PATH).to(|| async { HttpResponse::Ok() }))
                    .service(resource(INTERNAL_SERVER_ERROR_REQ_PATH).to(|| async { HttpResponse::Ok() })),
            )
            .await
        };
    }

    #[ntex::test]
    async fn not_found_path_normal() {
        let app = init_service!();

        let req = TestRequest::with_uri("/not-found-path").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(
            resp.response().headers().get(header::LOCATION).unwrap().to_str().unwrap(),
            vec![NOT_FOUND_REQ_PATH, "?", PREV_URL_SEARCH_QUERY_KEY, "=%2Fnot-found-path"].join("")
        );

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
        assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), "/not-found-path");
    }

    #[ntex::test]
    async fn not_found_path_normal_not_get() {
        let app = init_service!();

        let req = TestRequest::with_uri("/not-found-path").method(Method::POST).to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
    }

    #[ntex::test]
    async fn original_404_normal() {
        let app = init_service!();

        let req = TestRequest::with_uri(NOT_FOUND_REQ_PATH).to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
    }

    #[ntex::test]
    async fn not_found_path_from_json() {
        let app = init_service!();

        let mut req = TestRequest::with_uri("/not-found-path").to_request();

        req.headers_mut().insert(header::ACCEPT, HeaderValue::from_str(JSON_HEADER_VALUE).unwrap());

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());

        // Validate the body.
        let body_bytes = read_body(resp).await;
        let body: ServerResponse<String, &str> = serde_json::from_slice(&body_bytes).unwrap();
        // Status_code must be 200.
        // Cause we `skipped` the Serialization of the `status_code` property,
        // so we can only get the `Default 200` status_code value when Deserialization.
        assert_eq!(body, server_response_failed!(message: NOT_FOUND_MESSAGE, status_code: 200));
    }

    #[ntex::test]
    async fn not_found_path_from_ajax() {
        let app = init_service!();

        let mut req = TestRequest::with_uri("/not-found-path").to_request();

        req.headers_mut().insert(
            HeaderName::from_str(REQUESTED_WITH_HEADER_NAME).unwrap(),
            HeaderValue::from_str(REQUESTED_WITH_AJAX_HEADER_VALUE).unwrap(),
        );

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());

        // Validate the body.
        let body_bytes = read_body(resp).await;
        assert!(body_bytes.is_empty());
    }

    #[ntex::test]
    async fn internal_error_path_normal() {
        let app = init_service!();

        let req = TestRequest::with_uri("/test-500").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(
            resp.response().headers().get(header::LOCATION).unwrap().to_str().unwrap(),
            vec![INTERNAL_SERVER_ERROR_REQ_PATH, "?", PREV_URL_SEARCH_QUERY_KEY, "=%2Ftest-500"].join("")
        );

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
        assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), "/test-500");
    }

    #[ntex::test]
    async fn internal_error_path_normal_not_get() {
        let app = init_service!();

        let req = TestRequest::with_uri("/test-500").method(Method::POST).to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
    }

    #[ntex::test]
    async fn original_500_normal() {
        let app = init_service!();

        let req = TestRequest::with_uri(INTERNAL_SERVER_ERROR_REQ_PATH).to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
    }

    #[ntex::test]
    async fn internal_error_path_from_json() {
        let app = init_service!();

        let mut req = TestRequest::with_uri("/test-500").to_request();

        req.headers_mut().insert(header::ACCEPT, HeaderValue::from_str(JSON_HEADER_VALUE).unwrap());

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());

        // Validate the body.
        let body_bytes = read_body(resp).await;
        let body: ServerResponse<String, &str> = serde_json::from_slice(&body_bytes).unwrap();
        // Status_code must be 200.
        // Cause we `skipped` the Serialization of the `status_code` property,
        // so we can only get the `Default 200` status_code value when Deserialization.
        assert_eq!(body, server_response_failed!(message: INTERNAL_SERVER_ERROR_MESSAGE, status_code: 200));
    }

    #[ntex::test]
    async fn internal_error_path_from_ajax() {
        let app = init_service!();

        let mut req = TestRequest::with_uri("/test-500").to_request();

        req.headers_mut().insert(
            HeaderName::from_str(REQUESTED_WITH_HEADER_NAME).unwrap(),
            HeaderValue::from_str(REQUESTED_WITH_AJAX_HEADER_VALUE).unwrap(),
        );

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Validate extensions.
        assert!(resp.response().extensions().get::<OriginalUrl>().is_none());

        // Validate the body.
        let body_bytes = read_body(resp).await;
        assert!(body_bytes.is_empty());
    }
}
