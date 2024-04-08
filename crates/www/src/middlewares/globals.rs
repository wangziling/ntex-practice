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
                    let new_res = server_redirect!("/500")?;

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
                                Ok(query_map_string) => {
                                    *res.response_mut() =
                                        server_redirect!(uri.path().to_string() + "?" + &query_map_string)?;

                                    return Ok(res);
                                }
                                Err(_) => {}
                            }
                        }
                        _ => {}
                    }

                    *res.response_mut() = server_redirect!(uri.path().to_string())?;

                    return Ok(res);
                }
            }
            _ => {}
        }

        Ok(res)
    }
}
