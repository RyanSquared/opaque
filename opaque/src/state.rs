use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Author {
    pub(crate) name: String,
    pub(crate) email: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct FrontMatter {
    #[serde(default)]
    pub(crate) title: String,
    pub(crate) author: Option<Author>,
    pub(crate) date: Option<DateTime<Utc>>,
    pub(crate) published: Option<bool>,
}

impl FrontMatter {
    pub(crate) fn slug(&self) -> String {
        self.title.as_str().to_lowercase().replace(' ', "-")
    }
}

pub(crate) struct Page {
    pub(crate) front_matter: FrontMatter,
    pub(crate) file_path: PathBuf,
}

pub(crate) type PageMap = HashMap<String, Page>;

// TODO: This should be serde-able, and deserialized from a config file, similar to how
// Jekyll is configured. Information that goes here is typically information that will be
// displayed on the header and footer of every page. However, it will *not* include content such
// as a list of links to display in the nav bar.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: Author,
    pub(crate) url: String,
    pub(crate) static_path: PathBuf,
    pub(crate) content_path: PathBuf,
}

pub(crate) struct State {
    pub(crate) config: Config,
    pub(crate) page_map: Vec<(String, String)>,
    pub(crate) posts: PageMap,
}

impl State {
    pub(crate) fn new() -> State {
        State {
            config: Config {
                name: "Enigma".to_string(),
                description: "".to_string(),
                author: Author {
                    name: "RyanSquared".to_string(),
                    email: "me@ryansquared.pub".to_string(),
                },
                static_path: "static/".into(),
                content_path: "content/".into(),
                url: "https://ryansquared.pub/".to_string(),
            },
            page_map: vec![],
            posts: HashMap::new(),
        }
    }

    pub(crate) fn with_page_map(mut self, page_map: &[(String, String)]) -> Self {
        self.page_map = Vec::from(page_map);
        self
    }

    pub(crate) fn with_posts(mut self, posts: PageMap) -> Self {
        self.posts = posts;
        self
    }

    pub(crate) fn sorted_posts<'a>(&'a self) -> Vec<(&'a String, &'a Page)> {
        let mut posts = self
            .posts
            .iter()
            .filter(|v| v.1.front_matter.published.unwrap_or(true))
            .collect::<Vec<(&String, &Page)>>();
        posts.sort_by_key(|v| std::cmp::Reverse(v.1.front_matter.date));
        posts
    }
}
