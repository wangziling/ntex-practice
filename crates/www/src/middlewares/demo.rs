use web_core::middleware_prelude::*;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct SayHi;

impl<S> Middleware<S> for SayHi {
    type Service = SayHiInner<S>;

    fn create(&self, service: S) -> Self::Service {
        SayHiInner { service }
    }
}

pub struct SayHiInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for SayHiInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        println!("Hi from start. You requested: {}", req.uri());

        let res = ctx.call(&self.service, req).await?;

        println!("Hi from response.");

        Ok(res)
    }
}
