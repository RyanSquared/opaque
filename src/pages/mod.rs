use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup, DOCTYPE};

use crate::state::State;

pub(crate) mod assets;

mod components;

pub(crate) async fn index(state: Extension<Arc<State>>) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (components::head("Index"))
            body {
                (components::header(&state));
                main {
                    .content {
                        "Hello World!";
                    }
                }
                (components::footer(&state));
            }
        }
    }
}
