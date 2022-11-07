use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup};

use crate::state::State;

pub(crate) fn post_list(state: &Extension<Arc<State>>, post_limit: Option<usize>) -> Markup {
    let post_limit = post_limit.unwrap_or(usize::MAX);
    let posts = state.sorted_posts();
    html! {
        h1 { "Posts" }
        @for post in posts.iter().take(post_limit) {
            div.post {
                small {
                    @if let Some(date) = post.1.front_matter.date {
                        (date.format("%b %_d, %Y").to_string())
                        ", by ";
                        (post.1.front_matter.author
                         .as_ref()
                         .unwrap_or(&state.config.author)
                         .name)
                    } @else {
                        "By ";
                        (post.1.front_matter.author
                         .as_ref()
                         .unwrap_or(&state.config.author)
                         .name)
                    }
                }
                h2 { a href=(format!("/posts/{}", post.0)) { (post.1.front_matter.title) } }
            }
        }
    }
}
