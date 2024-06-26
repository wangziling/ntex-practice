use crate::models::controllers::HelloWorld;
use utoipa::ToSchema;
use web_cache::prelude::*;
use web_core::handler_prelude::*;

// #[instrument(skip_all)]
#[utoipa::path(
    get,
    path = "/greeting/hello",
    responses(
        (status = 200, description = "Hello world.", body = String),
    ),
)]
pub async fn hello(_req: HttpRequest, state: State<crate::app::AppState>) -> AppResult<&'static str> {
    // let distribute_cache = req.distribute_cache()?;
    // let _test_val = distribute_cache.get::<Option<String>, _>("test").await?;

    let _test_val = state.distribute_cache.get::<Option<String>, _>("test").await?;

    info!("Ip: {}, Port: {}", state.config.ip, state.config.port);

    Ok("Hello world!")
}

#[utoipa::path(
    get,
    path = "/greeting/hello2",
    responses(
        (status = 200, description = "Hello world.", body = ServerResponseHelloWorld),
        (status = 500, description = "Something wrong.", body = ServerResponseHelloWorld),
    ),
)]
pub async fn hello2(
    state: State<crate::app::AppState>,
    // _memory_cache: MemoryCacheExtension,
) -> AppResult<impl Responder> {
    // let memory_cache = memory_cache.read().await;
    let memory_cache = state.memory_cache.read().await;

    // Closure. Make sure we only insert once.
    memory_cache_make_sure!(memory_cache, {
        let _test_val = memory_cache
            .entry("test")
            .or_insert_with(async {
                info!("-----------------------------------------");

                json!(1)
            })
            .await;
    });

    info!(count = %memory_cache.entry_count()); // Must to be 1.

    Ok(server_response_success!(data: HelloWorld { greeting: "你好，世界。" }))
}

#[utoipa::path(
    get,
    path = "/greeting/hello3",
    request_body(content = Option<()>, description = "Json format", content_type = "application/json"),
    responses(
        (status = 300, description = "Hello world.", body = ServerResponseNullData),
    ),
)]
pub async fn hello3(req: HttpRequest, state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    // Make sure that only one process here on concurrent env now.
    // If you use `ab` bench tool to simulate concurrent request.
    // E.g. `ab -n 50 -c 10 http://localhost:5000/greeting/hello3`
    // Means concurrently request `hello3` with 10 client at the mean time and max requests are 500.
    // You will see that each second, here will only be one "-----" and one "22222" log.
    // Others requests are blocking.
    state
        .async_op_guard
        .spawn("AAA".as_bytes(), 1000, async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            info!(uri = %req.uri(), "-----------------");
        })
        .await?;

    info!("2222222222222");

    Ok(server_response_success!(status_code: 300))
}

#[utoipa::path(
    get,
    path = "/greeting/hello4",
    request_body(content = Option<()>, description = "Json format", content_type = "application/json"),
    responses(
        (status = 401, description = "Hello world.", body = ServerResponseNullData),
    ),
)]
pub async fn hello4(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 401))
}
