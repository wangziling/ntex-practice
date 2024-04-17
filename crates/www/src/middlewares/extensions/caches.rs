use std::sync::Arc;

use web_cache::prelude::*;
use web_core::middleware_prelude::*;

use crate::{app::AppState, utils::extensions};

pub struct PrepareCaches;

impl<S> Middleware<S> for PrepareCaches {
    type Service = PrepareCachesInner<S>;

    fn create(&self, service: S) -> Self::Service {
        PrepareCachesInner { service }
    }
}

pub struct PrepareCachesInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for PrepareCachesInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        {
            let mut extensions = req.extensions_mut();
            let app_state = req.app_state::<AppState>().unwrap();

            DistributeCacheExtension::set_into_req(&mut extensions, Arc::clone(&app_state.distribute_cache));
            MemoryCacheExtension::set_into_req(&mut extensions, Arc::clone(&app_state.memory_cache));
        }

        ctx.call(&self.service, req).await
    }
}
