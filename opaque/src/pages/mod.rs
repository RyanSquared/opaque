use std::sync::Arc;

use axum::Extension;
use chrono::{DateTime, Utc};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use serde::Deserialize;
use tracing::debug;

use opaque_markdown::render_path_to_html;

use crate::postprocessing::PostProcessingBuilder;
use crate::state::{Author, State};

pub(crate) mod assets;

#[derive(Clone, Debug, Default, Deserialize)]
struct FrontMatter {
    #[serde(default)]
    title: String,
    author: Option<Author>,
    date: Option<DateTime<Utc>>,
}

mod components;

#[tracing::instrument(skip(state))]
pub(crate) async fn index(state: Extension<Arc<State>>) -> Markup {
    let content_file = "content/posts/2022-09-23-an-inescapable-hell-of-networking.md";

    debug!("loading content from: {content_file}");
    let (content, front_matter_text) = render_path_to_html(content_file).await.expect("yike");

    debug!("decoding front matter");
    let mut front_matter = front_matter_text
        .map(|text| {
            let front_matter_str = text
                .as_str()
                .trim_start_matches("---\n")
                .trim_end()
                .trim_end_matches("---");
            match serde_yaml::from_str(front_matter_str) {
                Ok(result) => {
                    debug!(?result, "parsed front matter");
                    result
                }
                Err(bad_value) => {
                    debug!("unable to parse front matter: {bad_value}");
                    FrontMatter::default()
                }
            }
        })
        .unwrap_or_else(|| FrontMatter::default());

    front_matter.author = front_matter.author.or(Some(state.config.author.clone()));

    debug!("creating builder");
    let settings = PostProcessingBuilder::default()
        .rewrite_links("img[src]".to_string(), state.url.clone(), None)
        .expect("selector wasn't properly parsed")
        .convert_ansi(
            "opaque-ansi-output".to_string(),
            "output_snippets/".to_string(),
        )
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
                        h1 {
                            (front_matter.title)
                        }
                        @if let Some(date) = front_matter.date {
                            small {
                                (date.date().format("%b %_d, %Y").to_string())
                                @if let Some(author) = front_matter.author {
                                    ", by ";
                                    (author.name)
                                }
                            }
                        } @else if let Some(author) = front_matter.author {
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
    }
}
