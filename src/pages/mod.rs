use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::state::State;
use crate::posts::render_path_to_html;
use crate::rewrite_links::rewrite_links;

pub(crate) mod assets;

mod components;

pub(crate) async fn index(state: Extension<Arc<State>>) -> Markup {
    let content = render_path_to_html("content/posts/2022-09-23-an-inescapable-hell-of-networking.md").await.expect("yike");
    let content_rewritten = rewrite_links(content.as_ref(), state.url.as_ref()).expect("yoke");

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
