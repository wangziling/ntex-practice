use web_core::middleware_prelude::*;

pub struct RequireJson;

impl<S> Middleware<S> for RequireJson {
    type Service = RequireJsonInner<S>;

    fn create(&self, service: S) -> Self::Service {
        RequireJsonInner { service }
    }
}

pub struct RequireJsonInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for RequireJsonInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        if !req.derived_from_json() {
            return Err(crate::error::MiddlewareError::RequireJsonFormat.into());
        }

        ctx.call(&self.service, req).await
    }
}

pub struct ForAjaxReqOnly;

impl<S> Middleware<S> for ForAjaxReqOnly {
    type Service = ForAjaxReqOnlyInner<S>;

    fn create(&self, service: S) -> Self::Service {
        ForAjaxReqOnlyInner { service }
    }
}

pub struct ForAjaxReqOnlyInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for ForAjaxReqOnlyInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        if !req.derived_from_ajax() {
            return Err(crate::error::MiddlewareError::ForAjaxReqOnly.into());
        }

        ctx.call(&self.service, req).await
    }
}
