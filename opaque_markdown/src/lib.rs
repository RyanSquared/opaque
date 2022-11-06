use anyhow::Result;
use comrak::{format_html, nodes::{AstNode, NodeValue}, parse_document, Arena, ComrakOptions};

#[cfg(feature = "tracing")]
use tracing::debug;

use std::path::Path;

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

pub fn render_to_html(input: &str) -> Result<(String, Option<String>)> {
    // Create an arena for rendering purposes
    let arena = Arena::new();
    let root = parse_document(&arena, input, &COMRAK_OPTIONS);

    let mut front_matter: Option<String> = None;

    // note: this is fine since it's "front" matter.
    for node in root.children() {
        if let NodeValue::FrontMatter(v) = node.data.clone().into_inner().value {
            // note: we get passed a &str, it's not possible to get non-utf8
            front_matter.replace(String::from_utf8(v).unwrap());
        }
    }

    iter_nodes(root, &|node| match &mut node.data.borrow_mut().value {
        _ => ()
    });

    let mut html = vec![];
    format_html(root, &COMRAK_OPTIONS, &mut html)?;

    let string = String::from_utf8(html)?;
    Ok((string, front_matter))
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
#[cfg(feature = "tokio")]
pub async fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<(String, Option<String>)> {
    #[cfg(feature = "tracing")]
    debug!("reading file");

    let file_content = tokio::fs::read_to_string(&path).await?;

    #[cfg(feature = "tracing")]
    debug!("rendering HTML");

    render_to_html(file_content.as_str())
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
#[cfg(not(feature = "tokio"))]
pub async fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<(String, Option<String>)> {
    #[cfg(feature = "tracing")]
    debug!("reading file");
    
    let file_content = std::fs::read_to_string(&path)?;

    #[cfg(feature = "tracing")]
    debug!("rendering HTML");
    render_to_html(file_content.as_str())
}
