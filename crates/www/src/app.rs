use std::{ops::Deref, sync::Arc};
use web_core::prelude::*;

pub struct App {
    pub config: crate::config::Server,
    #[cfg(feature = "enable-redis")]
    pub distribute_cache: crate::cache::prelude::DistributeCacheGlobal,
}

impl App {
    pub async fn new(server_config: crate::config::Server) -> Result<Self> {
        Ok(App {
            #[cfg(feature = "enable-redis")]
            distribute_cache: crate::cache::generate_distribute_cache(server_config.distribute_cache_config.clone())
                .await?,
            config: server_config,
        })
    }
}

#[derive(Clone)]
pub struct AppState(pub Arc<App>);

impl Deref for AppState {
    type Target = Arc<App>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
