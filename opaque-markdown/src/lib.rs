use comrak::{
    format_html_with_plugins, nodes::AstNode, parse_document, Arena, ComrakOptions, ComrakPlugins,
};
use eyre::{Result, WrapErr};

#[cfg(feature = "tracing")]
use tracing::debug;

use std::path::Path;

mod syntect_adapter;

/// Create opinionated defaults for Comrak.
fn create_options() -> ComrakOptions {
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.header_ids = Some("md-header-".to_string());
    comrak_options.extension.description_lists = true;
    comrak_options.extension.front_matter_delimiter = Some("---".to_string());
    comrak_options.render.unsafe_ = true;
    comrak_options.render.unsafe_ = true;

    comrak_options
}

lazy_static::lazy_static! {
    static ref COMRAK_OPTIONS: ComrakOptions = create_options();
}

/// Call a given function for the current and every possible child of the Markdown node.
fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

/// Render a Markdown input to HTML using opinionated Comrak definitions.
///
/// # Errors
///
/// May arise from [`format_html_with_plugins`], returning a wrapped [`std::io::Error`].
pub fn render_to_html(input: &str) -> Result<String> {
    // Create an arena for rendering purposes
    let arena = Arena::new();
    let root = parse_document(&arena, input, &COMRAK_OPTIONS);

    #[allow(clippy::match_single_binding)]
    iter_nodes(root, &|node| match &mut node.data.borrow_mut().value {
        _ => (),
    });

    let mut comrak_plugins = ComrakPlugins::default();
    let syntax_adapter = syntect_adapter::SyntectAdapter::new();
    comrak_plugins.render.codefence_syntax_highlighter = Some(&syntax_adapter);

    let mut html = vec![];
    format_html_with_plugins(root, &COMRAK_OPTIONS, &mut html, &comrak_plugins)?;

    String::from_utf8(html).wrap_err("unable to decode html from utf8")
}

/// Load a file from the filesystem and render the the contents to HTML using opinionated Comrak
/// definitions.
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

/// Perform synchronous (blocking) functions.
pub mod sync {
    use super::{Path, Result, debug, render_to_html};

    /// Load a file from the filesystem and render the the contents to HTML using opinionated
    /// Comrak definitions.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<String> {
        #[cfg(feature = "tracing")]
        debug!("reading file");

        let file_content = std::fs::read_to_string(&path)?;

        #[cfg(feature = "tracing")]
        debug!("rendering HTML");
        render_to_html(file_content.as_str())
    }
}
