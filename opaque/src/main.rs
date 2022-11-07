use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

use axum::{routing::get, Extension, Router};

mod post_scanner;
mod pages;
mod postprocessing;
mod state;

fn setup_registry() {
    let envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();
    tracing_subscriber::registry()
        .with(envfilter)
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_registry();

    info!("everything becomes clear");

    // TODO: dynamic generation of either `app` or `page_map`?
    // I have not seen other projects do this so it may be fine to just leave it as-is. Besides,
    // this gives me the ability to add arbitrary URLs.
    
    let mut state = {
        let mut state = state::State::new();
        state
            .page_map
            .push(("About".to_string(), "/about".to_string()));
        state
    };

    post_scanner::walk_directory("content", &mut state.posts).await?;

    let app = Router::new()
        .route("/", get(pages::index))
        .route("/posts", get(pages::post::index))
        .route("/posts/:post", get(pages::post::slug))
        .route("/static/*path", get(pages::assets::static_path))
        .layer(CatchPanicLayer::new())
        .layer(Extension(Arc::new(state)))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:8000".parse()?;

    info!("serving on: {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
