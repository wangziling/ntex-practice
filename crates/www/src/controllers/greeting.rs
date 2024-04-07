pub use web_core::handler_prelude::*;

#[derive(Serialize)]
struct HelloWorld {
    greeting: &'static str,
}

// #[instrument(skip_all)]
pub async fn hello(state: State<crate::app::AppState>) -> &'static str {
    info!("Ip: {}, Port: {}", state.config.ip, state.config.port);

    "Hello world!"
}

#[get("/hello2")]
pub async fn hello2(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(Some(HelloWorld { greeting: "你好，世界。" })))
}

#[get("/hello3")]
pub async fn hello3(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 300.try_into().ok()))
}

pub async fn hello4(_state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    Ok(server_response_success!(status_code: 401.try_into().ok()))
}
