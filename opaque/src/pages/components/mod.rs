use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup};

use crate::state::State;

pub(crate) mod posts;

pub(crate) fn head(page_title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/static/assets/main.css";
            link rel="stylesheet" href="/static/assets/syntect.css";
            title {
                (page_title)
            }
        }
    }
}

pub(crate) fn header(state: &Extension<Arc<State>>) -> Markup {
    html! {
        header {
            .content {
                a #site_title href="/" {
                    (state.config.name);
                }

                nav #site_nav {
                    @for (title, route) in &state.page_map {
                        a href=(route) {
                            (title)
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn footer(state: &Extension<Arc<State>>) -> Markup {
    let mailto = format!("mailto:{}", state.config.author.email);
    html! {
        footer {
            .content {
                h2 {
                    (state.config.name);
                }
                // Partitioned: 30%, 25%, 45%
                div {
                    div {
                        ul.no_list_style {
                            li {
                                (state.config.author.name)
                            }
                            li {
                                a href=(mailto) {
                                    (state.config.author.email)
                                }
                            }
                        }
                    }
                    div {
                        ul.no_list_style;
                    }
                    div {
                        p {
                            (state.config.description)
                        }
                    }
                }
            }
        }
    }
}
