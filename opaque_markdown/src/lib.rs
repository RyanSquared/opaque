use anyhow::{Context, Result};
use comrak::{
    format_html_with_plugins, nodes::AstNode, parse_document, Arena, ComrakOptions, ComrakPlugins,
};

#[cfg(feature = "tracing")]
use tracing::debug;

use std::path::Path;

mod syntect_adapter;

fn create_options() -> ComrakOptions {
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.header_ids = Some("md-header-".to_string());
    comrak_options.extension.description_lists = true;
    // TODO: this should probably be parsed out and returned along with the renderd post
    comrak_options.extension.front_matter_delimiter = Some("---".to_string());
    comrak_options.render.unsafe_ = true;
    comrak_options.render.unsafe_ = true;

    comrak_options
}

lazy_static::lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = create_options();
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub fn render_to_html(input: &str) -> Result<String> {
    // Create an arena for rendering purposes
    let arena = Arena::new();
    let root = parse_document(&arena, input, &COMRAK_OPTIONS);

    iter_nodes(root, &|node| match &mut node.data.borrow_mut().value {
        _ => (),
    });

    let mut comrak_plugins = ComrakPlugins::default();
    let syntax_adapter = syntect_adapter::SyntectAdapter::new();
    comrak_plugins.render.codefence_syntax_highlighter = Some(&syntax_adapter);

    let mut html = vec![];
    format_html_with_plugins(root, &COMRAK_OPTIONS, &mut html, &comrak_plugins)?;

    String::from_utf8(html).context("unable to decode html from utf8")
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
#[cfg(feature = "tokio")]
pub async fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<String> {
    #[cfg(feature = "tracing")]
    debug!("reading file");

    let file_content = tokio::fs::read_to_string(&path).await?;

    #[cfg(feature = "tracing")]
    debug!("rendering HTML");

    render_to_html(file_content.as_str())
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
#[cfg(not(feature = "tokio"))]
pub async fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<String> {
    #[cfg(feature = "tracing")]
    debug!("reading file");

    let file_content = std::fs::read_to_string(&path)?;

    #[cfg(feature = "tracing")]
    debug!("rendering HTML");
    render_to_html(file_content.as_str())
}
