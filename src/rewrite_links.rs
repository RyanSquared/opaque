use lol_html::{rewrite_str, element, RewriteStrSettings};
use tracing::debug;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Unable to rewrite HTML")]
    RewriteHtmlFailure(#[from] lol_html::errors::RewritingError),
}

type Result<T> = std::result::Result<T, Error>;

#[tracing::instrument(skip(input))]
pub(crate) fn rewrite_links(input: &str, base_url: &str) -> Result<String> {
    debug!("rewriting page");

    let element_content_handlers = vec![
        element!("img[src]", |el| {
            let src = el
                .get_attribute("src")
                .expect("img[src] did not have src");

            let result = format!("{base_url}{src}");
            debug!("link found: {src}; rewriting: {result}");

            // the only thing that can cause an Error here is a memory alloc fail
            // which is out of scope
            el.set_attribute("src", result.as_ref()).unwrap();
            Ok(())
        })
    ];

    let result = rewrite_str(input, RewriteStrSettings {
        element_content_handlers,
        ..Default::default()
    })?;

    Ok(result)
}
