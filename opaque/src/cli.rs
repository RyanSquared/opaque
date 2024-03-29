use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::state::Author;

fn default_config_file() -> PathBuf {
    "config.yaml".into()
}

// This can be loaded from a configuration file and command line and then flattened into the
// resulting Config

/// An unsafe blog engine
#[derive(Clone, Debug, Default, Deserialize, Serialize, Parser)]
pub(crate) struct PartialConfig {
    /// The configuration file to load settings from
    #[arg(long, short, default_value = "config.yaml")]
    #[serde(default = "default_config_file")]
    pub(crate) config_file: PathBuf,

    /// The name that will be used for the blog
    #[arg(long)]
    pub(crate) name: Option<String>,

    /// The description posted in the footer
    #[arg(long)]
    pub(crate) description: Option<String>,

    /// The default author
    #[arg(long)]
    pub(crate) author: Option<Author>,

    /// The static URL of the hosted website
    #[arg(long)]
    pub(crate) url: Option<String>,

    /// The path of static content; this path will be appended to most resource URLs
    #[arg(long)]
    pub(crate) static_path: Option<PathBuf>,

    /// The address that the server will be bound to
    #[arg(long, short)]
    pub(crate) bind_address: Option<std::net::SocketAddr>,
}
