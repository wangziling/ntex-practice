use std::{ops::Deref, sync::Arc};
use web_core::prelude::*;

pub struct App {
    pub config: crate::config::Server,
}

impl App {
    pub async fn new(server_config: crate::config::Server) -> Result<Self> {
        Ok(App { config: server_config })
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
