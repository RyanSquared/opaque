use lol_html::{rewrite_str, element, RewriteStrSettings};

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Unable to rewrite HTML")]
    RewriteHtmlFailure(#[from] lol_html::errors::RewritingError),
}

type Result<T> = std::result::Result<T, Error>;

pub(crate) fn rewrite_links(input: &str, base_url: &str) -> Result<String> {
    let element_content_handlers = vec![
        element!("img[src]", |el| {
            let src = el
                .get_attribute("src")
                .expect("img[src] did not have src");

            // the only thing that can cause an Error here is a memory alloc fail
            // which is out of scope
            el.set_attribute("src", format!("{base_url}{src}").as_ref()).unwrap();
            Ok(())
        })
    ];

    Ok(rewrite_str(input, RewriteStrSettings {
        element_content_handlers,
        ..Default::default()
    })?)
}
