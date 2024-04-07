use std::sync::Arc;

use crate::cache::prelude::*;
use web_core::middleware_prelude::*;

pub struct ExtensionDistributeCache;

impl<S> Middleware<S> for ExtensionDistributeCache {
    type Service = ExtensionDistributeCacheInner<S>;

    fn create(&self, service: S) -> Self::Service {
        ExtensionDistributeCacheInner { service }
    }
}

pub struct ExtensionDistributeCacheInner<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for ExtensionDistributeCacheInner<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        // Using closure to avoid the borrow check of the `req`.
        {
            let mut extensions = req.extensions_mut();
            if !extensions.contains::<DistributeCacheExtension>() {
                let app_state = req.app_state::<crate::app::AppState>();
                match app_state {
                    Some(state) => {
                        extensions.insert(DistributeCacheExtension(Arc::clone(&state.distribute_cache)));
                    }
                    None => {
                        return Err(crate::error::MiddlewareError::AppStateMissing.into());
                    }
                }
            }
        }

        ctx.call(&self.service, req).await
    }
}
