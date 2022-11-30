use std::sync::Arc;

use color_eyre::{eyre::{Report, WrapErr}, Section};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

use axum::{routing::get, Extension, Router};

mod cli;
mod state;

mod pages;
mod post_scanner;
mod postprocessing;

fn setup_registry() {
    let envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();
    tracing_subscriber::registry()
        .with(envfilter)
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .with(tracing_error::ErrorLayer::default())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup_registry();

    color_eyre::install()?;

    // TODO: dynamic generation of either `app` or `page_map`?
    // I have not seen other projects do this so it may be fine to just leave it as-is. Besides,
    // this gives me the ability to add arbitrary URLs.

    let state = state::State::new_with_config_from_cli()
        .await
        .wrap_err("Unable to determine config from CLI or config file")?
        .with_page_map(&[("Posts".to_string(), "/posts".to_string())])
        .with_posts(
            post_scanner::walk_directory("content/posts")
                .await
                .wrap_err("Unable to load posts from posts directory")
                .suggestion("Run in Docker or Docker Compose?")?,
        );

    info!(?state.config, "Running with given configuration");

    let addr = state.config.bind_address;

    let app = Router::new()
        .route("/", get(pages::index))
        .route("/posts", get(pages::post::index))
        .route("/posts/:post", get(pages::post::slug))
        .route(
            format!(
                "/{}/*path",
                state
                    .config
                    .static_path
                    .display()
                    .to_string()
                    .trim_matches('/')
            )
            .as_str(),
            get(pages::assets::static_path),
        )
        .layer(CatchPanicLayer::new())
        .layer(Extension(Arc::new(state)))
        .layer(TraceLayer::new_for_http());

    info!("serving on: {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
