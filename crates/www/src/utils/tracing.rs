use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use web_core::prelude::*;

/// Copied from https://github.com/rust-lang/crates.io/blob/337923ea649195d0594b99b049b6bdfe92f67a0d/src/util/tracing.rs

fn init_with_default_level(default_level: LevelFilter, env_var: Option<String>) -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(default_level.into())
        .parse(env_var.unwrap_or_default())?;

    let log_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_line_number(true)
        .compact()
        .with_filter(env_filter);

    // Enable tracing.
    tracing_subscriber::registry().with(log_layer).init();

    Ok(())
}

pub fn init() -> Result<()> {
    init_with_default_level(LevelFilter::INFO, web_env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV)?)
}
