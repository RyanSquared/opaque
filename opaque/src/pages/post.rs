use std::sync::Arc;

use axum::{extract::Path, Extension};
use maud::{html, PreEscaped, DOCTYPE};
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use tracing::debug;

use opaque_markdown::render_path_to_html;

use super::{components, Error, Result};
use crate::postprocessing::PostProcessingBuilder;
use crate::state::State;

// Note: the cache doesn't need to be held across async yield boundaries, but tokio::sync::Mutex is
// still required over parking_lot::Mutex
static CACHE: OnceCell<Mutex<uluru::LRUCache<(String, String), 32>>> = OnceCell::new();

#[tracing::instrument(skip(state))]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub(crate) async fn index(state: Extension<Arc<State>>) -> Result {
    Ok(html! {
        (DOCTYPE)
        html {
            (components::head("Post Index"))
            body {
                (components::header(&state))
                main {
                    .content {
                        (components::posts::post_list(&state, None, None))
                    }
                }
            }
            (components::footer(&state))
        }
    })
}

#[tracing::instrument(skip(state, post_slug))]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub(crate) async fn slug(Path(post_slug): Path<String>, state: Extension<Arc<State>>) -> Result {
    let post = match state.posts.get(&post_slug) {
        Some(post) => post,
        None => return Err(Error::PostNotFound(post_slug)),
    };

    debug!(?post.front_matter.title, "found post for slug");

    let author = post
        .front_matter
        .author
        .as_ref()
        .unwrap_or(&state.config.author);

    let mut cache = CACHE
        .get_or_init(|| Mutex::new(uluru::LRUCache::default()))
        .lock()
        .await;

    let content = match cache.find(|(k, _)| post_slug == *k) {
        Some((_, hit)) => {
            debug!(?post_slug, "markdown: cache hit");
            hit.clone()
        }
        None => {
            let content = render_path_to_html(post.file_path.as_path()).await?;
            cache.insert((post_slug.clone(), content.clone()));
            content
        }
    };

    // NOTE: display() is lossy, need to figure out a way to ensure paths are UTF8.
    debug!("creating postprocessing builder");
    let settings = PostProcessingBuilder::default()
        .rewrite_links(
            "img[src]".to_string(),
            format!(
                "{}/{}",
                state.config.url.trim_matches('/'),
                state
                    .config
                    .static_path
                    .display()
                    .to_string()
                    .trim_start_matches('/')
            ),
            None,
        )
        .expect("selector wasn't properly parsed")
        .convert_ansi(
            "opaque-ansi-output".to_string(),
            "output_snippets/".to_string(),
            post_slug,
        )
        .unwrap()
        .build();

    debug!("rewriting content");
    let content_rewritten = lol_html::rewrite_str(content.as_str(), settings).expect("yoke");

    debug!("returning html body");
    Ok(html! {
        (DOCTYPE)
        html {
            (components::head(post.front_matter.title.as_str()))
            body {
                (components::header(&state));
                main {
                    .content {
                        h1 {
                            (post.front_matter.title)
                        }
                        @if let Some(date) = post.front_matter.date {
                            small {
                                (date.format("%b %_d, %Y").to_string())
                                ", by ";
                                (author.name)
                            }
                        } @else {
                            small {
                                "By ";
                                (author.name)
                            }
                        }
                        (PreEscaped(content_rewritten))
                    }
                }
                (components::footer(&state));
            }
        }
    })
}
