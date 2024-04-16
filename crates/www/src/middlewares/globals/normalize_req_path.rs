use crate::constants::{INTERNAL_SERVER_ERROR_REQ_PATH, NOT_FOUND_REQ_PATH};
use web_core::middleware_prelude::*;

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
    use ntex::util::{lazy, Bytes};
    use ntex::web::test::{init_service, read_body, TestRequest};
    use ntex::web::{resource, App, DefaultError, Error, HttpResponse};
    use once_cell::sync::Lazy;

    use web_core::constants::{JSON_HEADER_VALUE, REQUESTED_WITH_AJAX_HEADER_VALUE, REQUESTED_WITH_HEADER_NAME};
    use web_core::response::OriginalUrl;

    use super::NormalizeReqPath;

    const TEST_URL: &str = "/test";
    const TEST_PATHS: Lazy<Vec<&str>> = Lazy::new(|| vec!["/test/", "/test//////", "/test/?a=1", "/test//////?a=1"]);

    macro_rules! normal_works_well {
        ($app: expr) => {
            let req = TestRequest::with_uri(TEST_URL).to_request();
            let resp = $app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        };
    }

    #[ntex::test]
    async fn default() {
        let app = init_service(
            App::new()
                .wrap(NormalizeReqPath::default())
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
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
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
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
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
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
            let location_uri = Uri::from_str(resp.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
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
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
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
            let location_uri = Uri::from_str(resp.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
            assert_eq!(location_uri.path(), path_uri.path().trim_end_matches("/"));
            assert_eq!(location_uri.query(), path_uri.query());

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
            assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), *path);
        }
    }
}
