use crate::error::prelude::*;
use web_cache::prelude::*;
use web_core::handler_prelude::*;

#[derive(Serialize)]
struct HelloWorld {
    greeting: &'static str,
}

// #[instrument(skip_all)]
pub async fn hello(_req: HttpRequest, state: State<crate::app::AppState>) -> AppResult<&'static str> {
    // let extensions = req.extensions();
    // let distribute_cache = extensions
    //     .get::<DistributeCacheExtension>()
    //     .ok_or_else(|| crate::error::ExtensionError::DistributeCacheMissing.into_app_error())?;
    // let _test_val = distribute_cache.get::<Option<String>, _>("test").await?;

    let _test_val = state.distribute_cache.get::<Option<String>, _>("test").await?;

    info!("Ip: {}, Port: {}", state.config.ip, state.config.port);

    Ok("Hello world!")
}

pub async fn hello2(state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    let persistent_cache = state.persistent_cache.read().await;

    // Closure. Make sure we only insert once.
    persistent_mark_sure!(persistent_cache, {
        let _test_val = persistent_cache
            .entry("test")
            .or_insert_with(async {
                info!("-----------------------------------------");

                json!(1)
            })
            .await;
    });

    info!(count = %persistent_cache.entry_count()); // Must to be 1.

    Ok(server_response_success!(Some(HelloWorld { greeting: "你好，世界。" })))
}

pub async fn hello3(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 300))
}

pub async fn hello4(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 401))
}
