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
    type Error = Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let mut res = ctx.call(&self.service, req).await?;

        let req = res.request();
        match res.status() {
            StatusCode::INTERNAL_SERVER_ERROR if !req.path().eq("/500") => {
                if req.wants_json() {
                    *res.response_mut() = server_response_failed!(message: Some("Internal Server Error."), status_code: 500.try_into().ok()).into();

                    return Ok(res);
                }

                if !req.derived_from_ajax() {
                    let new_res = server_redirect!("/500", prev_url: Some(req.uri().to_string()))?;

                    {
                        let extensions = res.response().extensions();
                        let error_field = extensions.get::<ErrorField>();

                        if error_field.is_some() {
                            new_res.extensions_mut().insert(error_field.unwrap().clone());
                        }
                    }

                    *res.response_mut() = new_res;

                    return Ok(res);
                }
            }
            StatusCode::NOT_FOUND if !req.path().eq("/404") => {
                if req.wants_json() {
                    *res.response_mut() = server_response_failed!(message: Some("Requested resource not found."), status_code: 404.try_into().ok())
                    .into();

                    return Ok(res);
                }

                // UNWRAP: Operation must be successful.
                let mut uri = "/404".parse::<ntex::http::Uri>().unwrap();

                if !req.derived_from_ajax() && req.method() == Method::GET {
                    match req.uri().path_and_query() {
                        Some(path_query) => {
                            let query_map = uri
                                .update_query("prev".to_string(), Some(path_query.to_string()))
                                .map_err(Into::<BoxedAppError>::into)?;

                            match query_to_string(query_map) {
                                Ok(query_map_string) if !query_map_string.is_empty() => {
                                    *res.response_mut() = server_redirect!(uri.path().to_string() + "?" + &query_map_string, prev_url: Some(req.uri().to_string()))?;

                                    return Ok(res);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    *res.response_mut() =
                        server_redirect!(uri.path().to_string(), prev_url: Some(req.uri().to_string()))?;

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
    tailing_slash_mode: NormalizeReqPathTailingSlashMode,
}

impl<S> Middleware<S> for NormalizeReqPath {
    type Service = NormalizeReqPathInner<S>;

    fn create(&self, service: S) -> Self::Service {
        NormalizeReqPathInner { service, tailing_slash_mode: Default::default() }
    }
}

impl Default for NormalizeReqPath {
    fn default() -> Self {
        Self { tailing_slash_mode: Default::default() }
    }
}

pub struct NormalizeReqPathInner<S> {
    service: S,
    tailing_slash_mode: NormalizeReqPathTailingSlashMode,
}

impl<S, Err> Service<WebRequest<Err>> for NormalizeReqPathInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, mut req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        if self.tailing_slash_operation_enabled() {
            let head_mut = req.match_info_mut();
            let uri = head_mut.get_ref();
            let path = uri.path();

            if path.ends_with("/") {
                let origin_uri_string = uri.to_string();
                let transformed_path = path.trim_end_matches("/");

                let transformed_url = match uri.query() {
                    Some(query) if !query.is_empty() => transformed_path.to_string() + "?" + query,
                    _ => transformed_path.to_string(),
                };

                if self.tailing_slash_redirect() {
                    return Ok(server_redirect!(transformed_url, prev_url: Some(origin_uri_string), status_code: self.tailing_slash_redirect_status())?.into_web_response(req));
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
        pub fn use_tailing_slash_operation(mut self) -> Self {
            match self.tailing_slash_mode {
                NormalizeReqPathTailingSlashMode::LetItGo => {
                    self.tailing_slash_mode = NormalizeReqPathTailingSlashMode::NeedOperation {
                        use_redirect: false,
                        tailing_slash_redirect_status: None,
                    };
                }

                _ => {}
            }

            self
        }

        pub fn set_tailing_slash_redirect(mut self, use_redirect: bool) -> Self {
            self.tailing_slash_mode = match self.tailing_slash_mode {
                NormalizeReqPathTailingSlashMode::LetItGo => NormalizeReqPathTailingSlashMode::NeedOperation {
                    use_redirect,
                    tailing_slash_redirect_status: None,
                },
                NormalizeReqPathTailingSlashMode::NeedOperation { tailing_slash_redirect_status, .. } => {
                    NormalizeReqPathTailingSlashMode::NeedOperation { use_redirect, tailing_slash_redirect_status }
                }
            };

            self
        }

        pub fn set_tailing_slash_redirect_status<T: TryInto<StatusCode>>(
            mut self,
            tailing_slash_redirect_status: T,
        ) -> Self {
            match TryInto::<StatusCode>::try_into(tailing_slash_redirect_status).ok() {
                Some(tailing_slash_redirect_status) => {
                    self.tailing_slash_mode = match self.tailing_slash_mode {
                        NormalizeReqPathTailingSlashMode::LetItGo => NormalizeReqPathTailingSlashMode::NeedOperation {
                            use_redirect: true,
                            tailing_slash_redirect_status: Some(tailing_slash_redirect_status),
                        },
                        NormalizeReqPathTailingSlashMode::NeedOperation { tailing_slash_redirect_status, .. } => {
                            NormalizeReqPathTailingSlashMode::NeedOperation {
                                use_redirect: false,
                                tailing_slash_redirect_status,
                            }
                        }
                    };
                }

                _ => {}
            }

            self
        }

        pub fn tailing_slash_operation_enabled(&self) -> bool {
            match self.tailing_slash_mode {
                NormalizeReqPathTailingSlashMode::LetItGo => false,
                _ => true,
            }
        }

        pub fn tailing_slash_redirect(&self) -> bool {
            match self.tailing_slash_mode {
                NormalizeReqPathTailingSlashMode::NeedOperation { use_redirect, .. } => use_redirect,
                _ => false,
            }
        }

        pub fn tailing_slash_redirect_status(&self) -> Option<StatusCode> {
            match self.tailing_slash_mode {
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
