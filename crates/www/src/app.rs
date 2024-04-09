use std::{ops::Deref, sync::Arc};
use web_core::prelude::*;

pub struct App {
    pub config: crate::config::Server,
    pub distribute_cache: web_cache::prelude::DistributeCacheGlobal,
    pub persistent_cache: web_cache::prelude::PersistentCacheGlobal,
}

impl App {
    pub async fn new(server_config: crate::config::Server) -> Result<Self> {
        Ok(App {
            distribute_cache: web_cache::generate_distribute_cache(server_config.distribute_cache_config.clone())
                .await?,
            persistent_cache: Arc::clone(&web_cache::PERSISTENT_CACHE),
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
