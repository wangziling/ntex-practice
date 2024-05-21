use crate::constants::{INTERNAL_SERVER_ERROR_REQ_PATH, NOT_FOUND_REQ_PATH};
use once_cell::sync::Lazy;
use regex::Regex;
use web_core::middleware_prelude::*;

static MERGE_MULTIPLE_INTERIOR_SLASH_REGEXP: Lazy<Regex> = Lazy::new(|| Regex::new(r"//+").unwrap());

#[derive(Default, Clone)]
pub enum NormalizeReqPathInteriorSlashOpsMode {
    #[default]
    MergeMultiple,
}

#[derive(Default, Clone)]
pub enum NormalizeReqPathInteriorSlashOps {
    #[default]
    LetItGo,
    NeedOperation {
        mode: NormalizeReqPathInteriorSlashOpsMode,
    },
}

#[derive(Default)]
pub enum NormalizeReqPathSlashMode {
    #[default]
    LetItGo,
    NeedOperation {
        use_redirect: bool,
        redirect_status: Option<StatusCode>,
        interior_slash_ops: Option<NormalizeReqPathInteriorSlashOps>,
    },
}

#[derive(Default)]
pub struct NormalizeReqPath {
    slash_mode: std::rc::Rc<NormalizeReqPathSlashMode>,
}

impl<S> Middleware<S> for NormalizeReqPath {
    type Service = NormalizeReqPathInner<S>;

    fn create(&self, service: S) -> Self::Service {
        NormalizeReqPathInner { service, slash_mode: self.slash_mode.clone() }
    }
}

pub struct NormalizeReqPathInner<S> {
    service: S,
    slash_mode: std::rc::Rc<NormalizeReqPathSlashMode>,
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
        let mut path = uri.path();

        // The landing page.
        if path.is_empty() || path == "/" {
            return ctx.call(&self.service, req).await;
        }

        if self.slash_operation_enabled() {
            let mut is_path_changed = false;

            if self.interior_slash_ops_enabled() {
                if let Some(NormalizeReqPathInteriorSlashOpsMode::MergeMultiple) = self.interior_slash_ops() {
                    let result = MERGE_MULTIPLE_INTERIOR_SLASH_REGEXP.replace_all(path, "/").to_string();
                    if result != path {
                        path = result.leak();

                        // Mark as `changed`.
                        is_path_changed = true;
                    }
                }
            }

            macro_rules! operation {
                ($transformed_path: expr, $origin_uri_string: expr) => {
                    {
                        let transformed_url = match uri.query() {
                            Some(query) if !query.is_empty() => $transformed_path + "?" + query,
                            _ => $transformed_path,
                        };

                        if self.slash_redirect() {
                            return Ok(server_redirect!(transformed_url, prev_url: $origin_uri_string, optional_status_code: self.redirect_status())?.into_web_response(req));
                        }

                        head_mut.set(transformed_url.parse::<ntex::http::Uri>().map_err(Into::<BoxedAppError>::into)?);
                        req.head_mut().extensions_mut().insert(OriginalUrl::new($origin_uri_string));

                        ctx.call(&self.service, req).await
                    }
                };
            }

            let origin_uri_string = uri.to_string();
            if path.ends_with('/') {
                let mut transformed_path = path.trim_end_matches('/').to_owned();

                if transformed_path.is_empty() {
                    "/".clone_into(&mut transformed_path);
                }

                // Return.
                return operation!(transformed_path, origin_uri_string);
            }

            if is_path_changed {
                // Return.
                return operation!(path.to_owned(), origin_uri_string);
            }
        }

        ctx.call(&self.service, req).await
    }
}

macro_rules! __normalize_req_path_impl {
    () => {
        #[inline]
        fn wrap_slash_mode(mode: NormalizeReqPathSlashMode) -> std::rc::Rc<NormalizeReqPathSlashMode> {
            std::rc::Rc::new(mode)
        }

        pub fn use_slash_operation(mut self) -> Self {
            match *self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::LetItGo => {
                    self.slash_mode = Self::wrap_slash_mode(NormalizeReqPathSlashMode::NeedOperation {
                        use_redirect: false,
                        redirect_status: None,
                        interior_slash_ops: None,
                    });
                }

                _ => {}
            }

            self
        }

        pub fn set_slash_redirect(mut self, use_redirect: bool) -> Self {
            self.slash_mode = Self::wrap_slash_mode(match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::LetItGo => NormalizeReqPathSlashMode::NeedOperation {
                    use_redirect,
                    redirect_status: None,
                    interior_slash_ops: None,
                },
                NormalizeReqPathSlashMode::NeedOperation { redirect_status, interior_slash_ops, .. } => {
                    NormalizeReqPathSlashMode::NeedOperation {
                        use_redirect,
                        redirect_status: redirect_status.clone(),
                        interior_slash_ops: interior_slash_ops.clone(),
                    }
                }
            });

            self
        }

        pub fn set_redirect_status<T: TryInto<StatusCode>>(mut self, redirect_status: T) -> Self {
            match TryInto::<StatusCode>::try_into(redirect_status).ok() {
                Some(redirect_status) => {
                    self.slash_mode = Self::wrap_slash_mode(match self.slash_mode.as_ref() {
                        NormalizeReqPathSlashMode::LetItGo => NormalizeReqPathSlashMode::NeedOperation {
                            use_redirect: true,
                            redirect_status: Some(redirect_status),
                            interior_slash_ops: None,
                        },
                        NormalizeReqPathSlashMode::NeedOperation { use_redirect, interior_slash_ops, .. } => {
                            NormalizeReqPathSlashMode::NeedOperation {
                                use_redirect: use_redirect.clone(),
                                redirect_status: Some(redirect_status),
                                interior_slash_ops: interior_slash_ops.clone(),
                            }
                        }
                    });
                }

                _ => {}
            }

            self
        }

        pub fn enable_interior_slash_ops(mut self) -> Self {
            match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::NeedOperation { use_redirect, redirect_status, interior_slash_ops }
                    if interior_slash_ops.is_none()
                        || matches!(*interior_slash_ops, Some(NormalizeReqPathInteriorSlashOps::LetItGo)) =>
                {
                    self.slash_mode = Self::wrap_slash_mode(NormalizeReqPathSlashMode::NeedOperation {
                        use_redirect: use_redirect.clone(),
                        redirect_status: redirect_status.clone(),
                        interior_slash_ops: Some(NormalizeReqPathInteriorSlashOps::NeedOperation {
                            mode: Default::default(),
                        }),
                    });
                }

                _ => {}
            };

            self
        }

        #[inline]
        pub fn slash_operation_enabled(&self) -> bool {
            match *self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::LetItGo => false,
                _ => true,
            }
        }

        #[inline]
        pub fn slash_redirect(&self) -> bool {
            match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::NeedOperation { use_redirect, .. } => *use_redirect,
                _ => false,
            }
        }

        #[inline]
        pub fn redirect_status(&self) -> Option<&StatusCode> {
            match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::NeedOperation { redirect_status: Some(redirect_status), .. } => {
                    Some(redirect_status)
                }
                _ => None,
            }
        }

        #[inline]
        pub fn interior_slash_ops_enabled(&self) -> bool {
            match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::NeedOperation {
                    interior_slash_ops: Some(NormalizeReqPathInteriorSlashOps::NeedOperation { .. }),
                    ..
                } => true,
                _ => false,
            }
        }

        #[inline]
        pub fn interior_slash_ops(&self) -> Option<&NormalizeReqPathInteriorSlashOpsMode> {
            match self.slash_mode.as_ref() {
                NormalizeReqPathSlashMode::NeedOperation {
                    interior_slash_ops: Some(NormalizeReqPathInteriorSlashOps::NeedOperation { mode, .. }),
                    ..
                } => Some(mode),
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

    use super::{NormalizeReqPath, MERGE_MULTIPLE_INTERIOR_SLASH_REGEXP};

    const TEST_URL: &str = "/test/a/b/c";
    const TEST_PATHS: Lazy<Vec<&str>> =
        Lazy::new(|| vec!["/test/a/b/c/", "/test/a/b/c/////", "/test/a/b/c/?a=1", "/test/a/b/c/////?a=1"]);
    const TEST_PATHS_WITH_INTERIOR_SLASH: Lazy<Vec<&str>> = Lazy::new(|| {
        vec![
            "/test//////a///b/c/",
            "/test/a////b////c/////",
            "/test////a/b/c/?a=1",
            "/test/a/b////c/////?a=1",
            "/test/a/b////c?a=1",
        ]
    });

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
    async fn slash_default() {
        let app = init_service(
            App::new()
                .wrap(NormalizeReqPath::default().use_slash_operation())
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
    async fn slash_redirection() {
        let app = init_service(
            App::new()
                .wrap(NormalizeReqPath::default().use_slash_operation().set_slash_redirect(true))
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
    async fn slash_redirection_307() {
        let app = init_service(
            App::new()
                .wrap(
                    NormalizeReqPath::default()
                        .use_slash_operation()
                        .set_slash_redirect(true)
                        .set_redirect_status(StatusCode::TEMPORARY_REDIRECT),
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

    #[ntex::test]
    async fn interior_slash_ops() {
        let app = init_service(
            App::new()
                .wrap(NormalizeReqPath::default().use_slash_operation().enable_interior_slash_ops())
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
        )
        .await;

        // Works well.
        normal_works_well!(app);

        for path in TEST_PATHS_WITH_INTERIOR_SLASH.iter() {
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
    async fn interior_slash_ops_with_redirect() {
        let app = init_service(
            App::new()
                .wrap(
                    NormalizeReqPath::default()
                        .use_slash_operation()
                        .set_slash_redirect(true)
                        .enable_interior_slash_ops(),
                )
                .service(resource(TEST_URL).to(|| async { HttpResponse::Ok() })),
        )
        .await;

        // Works well.
        normal_works_well!(app);

        for path in TEST_PATHS_WITH_INTERIOR_SLASH.iter() {
            // Create request object
            let req = TestRequest::with_uri(*path).to_request();
            // Execute application
            let resp = app.call(req).await.unwrap();

            let path_uri = Uri::from_str(*path).unwrap();

            // Validate uri.
            let location_uri = Uri::from_str(resp.headers().get(header::LOCATION).unwrap().to_str().unwrap()).unwrap();
            assert_eq!(
                location_uri.path(),
                MERGE_MULTIPLE_INTERIOR_SLASH_REGEXP.replace_all(path_uri.path().trim_end_matches("/"), "/")
            );
            assert_eq!(location_uri.query(), path_uri.query());

            // Validate extensions.
            assert!(resp.response().extensions().get::<OriginalUrl>().is_some());
            assert_eq!(resp.response().extensions().get::<OriginalUrl>().unwrap().as_str(), *path);
        }
    }
}
