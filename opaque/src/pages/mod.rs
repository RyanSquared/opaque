use std::sync::Arc;

use axum::{
    body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

use opaque_markdown::render_path_to_html;

use crate::state::State;

pub(crate) mod assets;
pub(crate) mod components;

pub(crate) mod post;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("blog post not found: {0}")]
    PostNotFound(String),

    #[error("the server had an internal error: {0}")]
    InternalServerError(String),
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

#[tracing::instrument(skip(state))]
pub(crate) async fn index(state: Extension<Arc<State>>) -> Result<Markup> {
    let path = state.config.content_path.join("about.md");
    let content = render_path_to_html(path).await?;
    Ok(html! {
        (DOCTYPE)
        html {
            (components::head("Index"))
            body {
                (components::header(&state))
                main {
                    .content {
                        (components::posts::post_list(&state, Some(5), Some("Recent Posts")))
                        (PreEscaped(content))
                    }
                }
                (components::footer(&state))
            }
        }
    })
}
