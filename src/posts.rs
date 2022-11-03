use std::path::Path;

use comrak::{
    parse_document, format_html, Arena, ComrakOptions,
    nodes::AstNode,
};
use tracing::debug;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("markdown formatting failed: {0}")]
    MarkdownFormat(std::io::Error),

    #[error("invalid UTF-8: {0}")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),

    #[error("File read error: {0}, context?: {1}")]
    FileRead(std::io::Error, String)
}

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

pub(crate) type Result<T> = std::result::Result<T, Error>;

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where F: Fn(&'a AstNode<'a>) {
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub(crate) fn render_to_html(input: &str) -> Result<String> {
    // Create an arena for rendering purposes
    let arena = Arena::new();
    let root = parse_document(&arena, input, &COMRAK_OPTIONS);

    iter_nodes(root, &|node| {
        match &mut node.data.borrow_mut().value {
            _ => (),
        }
    });

    let mut html = vec![];
    if let Err(e) = format_html(root, &COMRAK_OPTIONS, &mut html) {
        return Err(Error::MarkdownFormat(e))
    }

    let string = String::from_utf8(html)?;
    Ok(string)
}

#[tracing::instrument]
pub(crate) async fn render_path_to_html(path: impl AsRef<Path> + std::fmt::Debug) -> Result<String> {
    debug!("reading file");
    let file_content = match tokio::fs::read_to_string(&path).await {
        Ok(content) => content,
        Err(io_err) => return Err(Error::FileRead(io_err, format!("path: {path:?}"))),
    };

    debug!("rendering HTML");
    render_to_html(file_content.as_str())
}
