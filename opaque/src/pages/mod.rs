use std::sync::Arc;

use axum::{
    body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use maud::{html, Markup};

use crate::state::State;

pub(crate) mod assets;
pub(crate) mod components;

pub(crate) mod post;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("blog post not found: {0}")]
    PostNotFound(String),

    #[error("the server had an internal error: {0}")]
    InternalServerError(String)
}

pub(crate) type Result<T = Markup> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalServerError(error.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let response_body = body::boxed(body::Full::from(self.to_string()));
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(response_body)
            .unwrap()
    }
}

#[tracing::instrument(skip(_state))]
pub(crate) async fn index(_state: Extension<Arc<State>>) -> Markup {
    html! {}
}
