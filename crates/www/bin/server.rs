use std::sync::Arc;
use web_core::prelude::*;

#[ntex::main]
async fn main() -> Result<()> {
    web_www::utils::tracing::init()?;

    let server_config = web_www::config::Server::from_env()?;
    let server_bind = (server_config.ip, server_config.port);
    let app = Arc::new(web_www::app::App::new(server_config).await?);

    let server = ntex::web::HttpServer::new(move || {
        ntex::web::App::new()
            .wrap(web_www::middlewares::globals::Centralization)
            .wrap(
                web_www::middlewares::globals::NormalizeReqPath::default()
                    .use_slash_operation()
                    .set_slash_redirect(true)
                    .set_redirect_status(301)
                    .enable_interior_slash_ops(),
            )
            // .wrap(web_www::middlewares::extensions::PrepareCaches)
            .wrap(ntex::web::middleware::Compress::default())
            .wrap(ntex::web::middleware::DefaultHeaders::new().header("X-Powered-By", "ntex-rs"))
            .state(web_www::app::AppState(app.clone()))
            .configure(web_www::routes::build_routes)
    });

    #[cfg(feature = "tls-rustls")]
    let server = server.bind_rustls(
        server_bind,
        web_www::utils::server::generate_tls_rustls_config(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("self_signed_certs/key.pem"),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("self_signed_certs/cert.pem"),
        )?,
    )?;

    #[cfg(not(feature = "tls-rustls"))]
    let server = server.bind(server_bind)?;

    server.run().await?;

    Ok(())
}
