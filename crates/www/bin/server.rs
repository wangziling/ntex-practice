use std::sync::Arc;
use web_core::prelude::*;

#[ntex::main]
async fn main() -> Result<()> {
    web_www::utils::tracing::init()?;

    let server_config = web_www::config::Server::from_env()?;
    let server_bind = (server_config.ip.clone(), server_config.port.clone());
    let app = Arc::new(web_www::app::App::new(server_config).await?);

    ntex::web::HttpServer::new(move || {
        ntex::web::App::new()
            .wrap(web_www::middlewares::globals::Centralization)
            .wrap(ntex::web::middleware::Compress::default())
            .wrap(ntex::web::middleware::DefaultHeaders::new().header("X-Powered-By", "ntex-rs"))
            .state(web_www::app::AppState(app.clone()))
            .configure(web_www::routes::build_routes)
    })
    .bind(server_bind)?
    .run()
    .await?;

    Ok(())
}
