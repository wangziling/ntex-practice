use web_core::middleware_prelude::*;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
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
            StatusCode::INTERNAL_SERVER_ERROR if !req.path().eq("/500") => {
                if req.wants_json() {
                    *res.response_mut() =
                        server_response_failed!(message: "Internal Server Error.", status_code: 500).into();

                    return Ok(res);
                }

                if !req.derived_from_ajax() && req.method() == Method::GET {
                    // UNWRAP: Operation must be successful.
                    let mut uri = "/500".parse::<ntex::http::Uri>().unwrap();

                    match req.uri().path_and_query() {
                        Some(path_query) => {
                            let query_map = uri
                                .update_query("prev".to_string(), Some(path_query.to_string()))
                                .map_err(Into::<BoxedAppError>::into)?;

                            match query_to_string(query_map) {
                                Ok(query_map_string) if !query_map_string.is_empty() => {
                                    *res.response_mut() = server_redirect!(uri.path().to_string() + "?" + &query_map_string, prev_url: req.uri().to_string())?;

                                    return Ok(res);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    *res.response_mut() = server_redirect!(uri.path().to_string(), prev_url: req.uri().to_string())?;

                    return Ok(res);
                }
            }
            StatusCode::NOT_FOUND if !req.path().eq("/404") => {
                if req.wants_json() {
                    *res.response_mut() =
                        server_response_failed!(message: "Requested resource not found.", status_code: 404).into();

                    return Ok(res);
                }

                if !req.derived_from_ajax() && req.method() == Method::GET {
                    // UNWRAP: Operation must be successful.
                    let mut uri = "/404".parse::<ntex::http::Uri>().unwrap();

                    match req.uri().path_and_query() {
                        Some(path_query) => {
                            let query_map = uri
                                .update_query("prev".to_string(), Some(path_query.to_string()))
                                .map_err(Into::<BoxedAppError>::into)?;

                            match query_to_string(query_map) {
                                Ok(query_map_string) if !query_map_string.is_empty() => {
                                    *res.response_mut() = server_redirect!(uri.path().to_string() + "?" + &query_map_string, prev_url: req.uri().to_string())?;

                                    return Ok(res);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    *res.response_mut() = server_redirect!(uri.path().to_string(), prev_url: req.uri().to_string())?;

                    return Ok(res);
                }
            }
            _ => {}
        }

        Ok(res)
    }
}

#[derive(Default)]
pub enum NormalizeReqPathTailingSlashMode {
    #[default]
    LetItGo,
    NeedOperation {
        use_redirect: bool,
        tailing_slash_redirect_status: Option<StatusCode>,
    },
}

pub struct NormalizeReqPath {
    tailing_slash_mode: std::rc::Rc<NormalizeReqPathTailingSlashMode>,
}

impl<S> Middleware<S> for NormalizeReqPath {
    type Service = NormalizeReqPathInner<S>;

    fn create(&self, service: S) -> Self::Service {
        NormalizeReqPathInner { service, tailing_slash_mode: self.tailing_slash_mode.clone() }
    }
}

impl Default for NormalizeReqPath {
    fn default() -> Self {
        Self { tailing_slash_mode: Default::default() }
    }
}

pub struct NormalizeReqPathInner<S> {
    service: S,
    tailing_slash_mode: std::rc::Rc<NormalizeReqPathTailingSlashMode>,
}

impl<S, Err> Service<WebRequest<Err>> for NormalizeReqPathInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, mut req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let head_mut = req.match_info_mut();
        let uri = head_mut.get_ref();
        let path = uri.path();

        // The landing page.
        if path.is_empty() || path == "/" {
            return ctx.call(&self.service, req).await;
        }

        if self.tailing_slash_operation_enabled() {
            if path.ends_with("/") {
                let origin_uri_string = uri.to_string();
                let mut transformed_path = path.trim_end_matches("/").to_string();

                if transformed_path.is_empty() {
                    transformed_path = "/".to_owned();
                }

                let transformed_url = match uri.query() {
                    Some(query) if !query.is_empty() => transformed_path + "?" + query,
                    _ => transformed_path,
                };

                if self.tailing_slash_redirect() {
                    return Ok(server_redirect!(transformed_url, prev_url: origin_uri_string, optional_status_code: self.tailing_slash_redirect_status())?.into_web_response(req));
                }

                head_mut.set(transformed_url.parse::<ntex::http::Uri>().map_err(Into::<BoxedAppError>::into)?);
                req.head_mut().extensions_mut().insert(OriginalUrl::new(origin_uri_string));
            }
        }

        ctx.call(&self.service, req).await
    }
}

macro_rules! __normalize_req_path_impl {
    () => {
        fn wrap_tailing_slash_mode(
            mode: NormalizeReqPathTailingSlashMode,
        ) -> std::rc::Rc<NormalizeReqPathTailingSlashMode> {
            std::rc::Rc::new(mode)
        }

        pub fn use_tailing_slash_operation(mut self) -> Self {
            match *self.tailing_slash_mode.as_ref() {
                NormalizeReqPathTailingSlashMode::LetItGo => {
                    self.tailing_slash_mode =
                        Self::wrap_tailing_slash_mode(NormalizeReqPathTailingSlashMode::NeedOperation {
                            use_redirect: false,
                            tailing_slash_redirect_status: None,
                        });
                }

                _ => {}
            }

            self
        }

        pub fn set_tailing_slash_redirect(mut self, use_redirect: bool) -> Self {
            self.tailing_slash_mode = Self::wrap_tailing_slash_mode(match *self.tailing_slash_mode.as_ref() {
                NormalizeReqPathTailingSlashMode::LetItGo => NormalizeReqPathTailingSlashMode::NeedOperation {
                    use_redirect,
                    tailing_slash_redirect_status: None,
                },
                NormalizeReqPathTailingSlashMode::NeedOperation { tailing_slash_redirect_status, .. } => {
                    NormalizeReqPathTailingSlashMode::NeedOperation { use_redirect, tailing_slash_redirect_status }
                }
            });

            self
        }

        pub fn set_tailing_slash_redirect_status<T: TryInto<StatusCode>>(
            mut self,
            tailing_slash_redirect_status: T,
        ) -> Self {
            match TryInto::<StatusCode>::try_into(tailing_slash_redirect_status).ok() {
                Some(tailing_slash_redirect_status) => {
                    self.tailing_slash_mode = Self::wrap_tailing_slash_mode(match *self.tailing_slash_mode.as_ref() {
                        NormalizeReqPathTailingSlashMode::LetItGo => NormalizeReqPathTailingSlashMode::NeedOperation {
                            use_redirect: true,
                            tailing_slash_redirect_status: Some(tailing_slash_redirect_status),
                        },
                        NormalizeReqPathTailingSlashMode::NeedOperation { use_redirect, .. } => {
                            NormalizeReqPathTailingSlashMode::NeedOperation {
                                use_redirect,
                                tailing_slash_redirect_status: Some(tailing_slash_redirect_status),
                            }
                        }
                    });
                }

                _ => {}
            }

            self
        }

        pub fn tailing_slash_operation_enabled(&self) -> bool {
            match *self.tailing_slash_mode.as_ref() {
                NormalizeReqPathTailingSlashMode::LetItGo => false,
                _ => true,
            }
        }

        pub fn tailing_slash_redirect(&self) -> bool {
            match *self.tailing_slash_mode.as_ref() {
                NormalizeReqPathTailingSlashMode::NeedOperation { use_redirect, .. } => use_redirect,
                _ => false,
            }
        }

        pub fn tailing_slash_redirect_status(&self) -> Option<StatusCode> {
            match *self.tailing_slash_mode.as_ref() {
                NormalizeReqPathTailingSlashMode::NeedOperation { tailing_slash_redirect_status, .. } => {
                    tailing_slash_redirect_status
                }
                _ => None,
            }
        }
    };
}

macro_rules! normalize_req_path_impl {
    ($ident: ident) => {
        impl $ident {
            __normalize_req_path_impl!();
        }
    };

    ($ident: ident<$($generic: tt),+>) => {
        impl<$($generic)+> $ident<$($generic)+> {
            __normalize_req_path_impl!();
        }
    };
}
normalize_req_path_impl!(NormalizeReqPath);
normalize_req_path_impl!(NormalizeReqPathInner<S>);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ntex::http::{header, header::HeaderName, header::HeaderValue, StatusCode, Uri};
    use ntex::service::{IntoService, Middleware, Pipeline};
    use ntex::util::lazy;
    use ntex::web::test::{init_service, TestRequest};
    use ntex::web::{resource, App, DefaultError, Error, HttpResponse};
    use once_cell::sync::Lazy;

    use super::{Centralization, NormalizeReqPath, OriginalUrl};

    #[cfg(test)]
    mod centralization {
        use super::*;

        macro_rules! init_service {
            () => {
                init_service(
                    App::new()
                        .wrap(Centralization)
                        .service(resource("/test").to(|| async { HttpResponse::Ok() }))
                        .service(
                            resource("/test-500").to(|| async { HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR) }),
                        )
                        .service(resource("/404").to(|| async { HttpResponse::Ok() }))
                        .service(resource("/500").to(|| async { HttpResponse::Ok() })),
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
                "/404?prev=%2Fnot-found-path"
            );

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
            assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), "/not-found-path");
        }

        #[ntex::test]
        async fn original_404_normal() {
            let app = init_service!();

            let req = TestRequest::with_uri("/404").to_request();

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }

        #[ntex::test]
        async fn not_found_path_from_json() {
            let app = init_service!();

            let mut req = TestRequest::with_uri("/not-found-path").to_request();

            req.headers_mut().insert(header::ACCEPT, HeaderValue::from_str("application/json").unwrap());

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }

        #[ntex::test]
        async fn not_found_path_from_ajax() {
            let app = init_service!();

            let mut req = TestRequest::with_uri("/not-found-path").to_request();

            req.headers_mut().insert(
                HeaderName::from_str("x-requested-with").unwrap(),
                HeaderValue::from_str("XMLHttpRequest").unwrap(),
            );

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }

        #[ntex::test]
        async fn internal_error_path_normal() {
            let app = init_service!();

            let req = TestRequest::with_uri("/test-500").to_request();

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::FOUND);
            assert_eq!(
                resp.response().headers().get(header::LOCATION).unwrap().to_str().unwrap(),
                "/500?prev=%2Ftest-500"
            );

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
            assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), "/test-500");
        }

        #[ntex::test]
        async fn original_500_normal() {
            let app = init_service!();

            let req = TestRequest::with_uri("/500").to_request();

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }

        #[ntex::test]
        async fn internal_error_path_from_json() {
            let app = init_service!();

            let mut req = TestRequest::with_uri("/test-500").to_request();

            req.headers_mut().insert(header::ACCEPT, HeaderValue::from_str("application/json").unwrap());

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }

        #[ntex::test]
        async fn internal_error_path_from_ajax() {
            let app = init_service!();

            let mut req = TestRequest::with_uri("/test-500").to_request();

            req.headers_mut().insert(
                HeaderName::from_str("x-requested-with").unwrap(),
                HeaderValue::from_str("XMLHttpRequest").unwrap(),
            );

            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_none());
        }
    }

    #[cfg(test)]
    mod normalize_req_path {
        use super::*;

        const TEST_PATHS: Lazy<Vec<&str>> =
            Lazy::new(|| vec!["/test/", "/test//////", "/test/?a=1", "/test//////?a=1"]);

        macro_rules! normal_works_well {
            ($app: expr) => {
                let req = TestRequest::with_uri("/test").to_request();
                let resp = $app.call(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
            };
        }

        #[ntex::test]
        async fn default() {
            let app = init_service(
                App::new()
                    .wrap(NormalizeReqPath::default())
                    .service(resource("/test").to(|| async { HttpResponse::Ok() })),
            )
            .await;

            // Works well.
            normal_works_well!(app);

            for path in TEST_PATHS.iter() {
                // Create request object
                let req = TestRequest::with_uri(*path).to_request();
                // Execute application
                let resp = app.call(req).await.unwrap();

                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            }
        }

        #[ntex::test]
        async fn tailing_slash_default() {
            let app = init_service(
                App::new()
                    .wrap(NormalizeReqPath::default().use_tailing_slash_operation())
                    .service(resource("/test").to(|| async { HttpResponse::Ok() })),
            )
            .await;

            // Works well.
            normal_works_well!(app);

            for path in TEST_PATHS.iter() {
                // Create request object
                let req = TestRequest::with_uri(*path).to_request();
                // Execute application
                let resp = app.call(req).await.unwrap();

                let path_uri = Uri::from_str(*path).unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
                assert_eq!(*resp.request().uri(), path_uri);
                assert!(resp.request().extensions().get::<OriginalUrl>().is_some());
                assert_eq!(resp.request().extensions().get::<OriginalUrl>().unwrap().as_str(), *path);
            }
        }

        #[ntex::test]
        async fn tailing_slash_redirection() {
            let app = init_service(
                App::new()
                    .wrap(NormalizeReqPath::default().use_tailing_slash_operation().set_tailing_slash_redirect(true))
                    .service(resource("/test").to(|| async { HttpResponse::Ok() })),
            )
            .await;

            // Works well.
            normal_works_well!(app);

            for path in TEST_PATHS.iter() {
                // Create request object
                let req = TestRequest::with_uri(*path).to_request();
                // Execute application
                let resp = app.call(req).await.unwrap();

                let path_uri = Uri::from_str(*path).unwrap();

                // Validate status.
                assert_eq!(resp.status(), StatusCode::FOUND);

                // Validate uri.
                let location_uri =
                    Uri::from_str(resp.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
                assert_eq!(location_uri.path(), path_uri.path().trim_end_matches("/"));
                assert_eq!(location_uri.query(), path_uri.query());

                // Validate extensions.
                assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
                assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), *path);
            }
        }

        #[ntex::test]
        async fn tailing_slash_redirection_307() {
            let app = init_service(
                App::new()
                    .wrap(
                        NormalizeReqPath::default()
                            .use_tailing_slash_operation()
                            .set_tailing_slash_redirect(true)
                            .set_tailing_slash_redirect_status(StatusCode::TEMPORARY_REDIRECT),
                    )
                    .service(resource("/test").to(|| async { HttpResponse::Ok() })),
            )
            .await;

            // Works well.
            normal_works_well!(app);

            for path in TEST_PATHS.iter() {
                // Create request object
                let req = TestRequest::with_uri(*path).to_request();
                // Execute application
                let resp = app.call(req).await.unwrap();

                let path_uri = Uri::from_str(*path).unwrap();

                // Validate status.
                assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);

                // Validate uri.
                let location_uri =
                    Uri::from_str(resp.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
                assert_eq!(location_uri.path(), path_uri.path().trim_end_matches("/"));
                assert_eq!(location_uri.query(), path_uri.query());

                // Validate extensions.
                assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
                assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), *path);
            }
        }
    }
}
