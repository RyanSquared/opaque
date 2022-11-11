use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

use crate::cli::PartialConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Author {
    pub(crate) name: String,
    pub(crate) email: String,
}

impl std::str::FromStr for Author {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ryan Heywood <ryan@hashbang.sh>
        let splice_index = s.find('<').ok_or(format!("could not email in {}", s))?;
        let name = &s[..splice_index].trim();
        let email = &s[splice_index + 1..].trim_matches(&['<', '>'][..]);
        Ok(Author { name: name.to_string(), email: email.to_string() })
    }
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

// NOTE: Any field changed here should be changed in opaque::cli::PartialConfig
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: Author,
    pub(crate) url: String,
    pub(crate) static_path: PathBuf,
    pub(crate) bind_address: std::net::SocketAddr,
}

pub(crate) struct State {
    pub(crate) config: Config,
    pub(crate) page_map: Vec<(String, String)>,
    pub(crate) posts: PageMap,
}

impl State {
    #[allow(dead_code)]
    #[deprecated(since="0.1.0", note="use new_with_config_from_cli instead")]
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
                url: "https://ryansquared.pub/".to_string(),
                bind_address: "0.0.0.0:8000".parse().expect("couldn't parse static address"),
            },
            page_map: vec![],
            posts: HashMap::new(),
        }
    }

    pub(crate) async fn new_with_config_from_cli() -> Result<Self> {
        let config_file = PartialConfig::parse().config_file;
        let mut config_object: PartialConfig = if config_file.exists() {
            let config_text = read_to_string(config_file).await?;
             serde_yaml::from_str(config_text.as_str())?
        } else {
            Default::default()
        };

        // takes priority
        // note: optimizing std::env::args_os() is not significant since std::env::args_os()
        // returns impl Iterator
        config_object.update_from(std::env::args_os());

        // convert from a PartialConfig to a Config, turning Option<T> into T
        let config: Config = serde_yaml::from_value(serde_yaml::to_value(config_object)?)?;

        Ok(State {
            // serialize to value, deserialize from value
            config,
            page_map: vec![],
            posts: HashMap::new(),
        })
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
