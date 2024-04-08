use crate::{cache::prelude::*, error::prelude::*};
use web_core::handler_prelude::*;

#[derive(Serialize)]
struct HelloWorld {
    greeting: &'static str,
}

// #[instrument(skip_all)]
pub async fn hello(req: HttpRequest, state: State<crate::app::AppState>) -> AppResult<&'static str> {
    let extensions = req.extensions();
    let distribute_cache = extensions
        .get::<DistributeCacheExtension>()
        .ok_or_else(|| crate::error::ExtensionError::DistributeCacheMissing.into_app_error())?;

    let _test_val = distribute_cache.get::<Option<String>, _>("test").await?;

    info!("Ip: {}, Port: {}", state.config.ip, state.config.port);

    Ok("Hello world!")
}

pub async fn hello2(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(Some(HelloWorld { greeting: "你好，世界。" })))
}

pub async fn hello3(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 300.try_into().ok()))
}

pub async fn hello4(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 401.try_into().ok()))
}
