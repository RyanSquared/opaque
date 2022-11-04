use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use tracing::debug;

use opaque_markdown::render_path_to_html;

use crate::postprocessing::PostProcessingBuilder;
use crate::state::State;

pub(crate) mod assets;

mod components;

#[tracing::instrument(skip(state))]
pub(crate) async fn index(state: Extension<Arc<State>>) -> Markup {
    let content_file = "content/posts/2022-09-23-an-inescapable-hell-of-networking.md";

    debug!("loading content from: {content_file}");
    let content = render_path_to_html(content_file).await.expect("yike");

    debug!("creating builder");
    let settings = PostProcessingBuilder::default()
        .rewrite_links("img[src]".to_string(), state.url.clone(), None)
        .expect("selector wasn't properly parsed")
        .convert_ansi("opaque-ansi-output".to_string(), "output_snippets/".to_string())
        .unwrap()
        .build();

    debug!("rewriting content");
    let content_rewritten = lol_html::rewrite_str(content.as_str(), settings).expect("yoke");

    debug!("returning html body");
    html! {
        (DOCTYPE)
        html {
            (components::head("Index"))
            body {
                (components::header(&state));
                main {
                    .content {
                        (PreEscaped(content_rewritten))
                    }
                }
                (components::footer(&state));
            }
        }
    }
}
