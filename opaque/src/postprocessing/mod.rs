use lol_html::{element, RewriteStrSettings};

mod rewrite_links;
use rewrite_links::RewriteLinks;

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("A selector was neither parsed from .attribute nor found when initialized")]
    InvalidSelector,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub(crate) struct PostProcessingBuilder {
    rewrite_links_selector: Vec<(String, RewriteLinks)>,
}

impl PostProcessingBuilder {
    pub(crate) fn rewrite_links(
        mut self,
        selector: String,
        url: String,
        attribute: Option<String>,
    ) -> Result<Self> {
        let attribute = attribute
            .or_else(|| selector.split("[").nth(1).map(|v| String::from(v.trim_end_matches(']'))))
            .ok_or(Error::InvalidSelector);
        dbg!(&selector, &url, &attribute);
        let rewrite_links = RewriteLinks::new(url, attribute?);
        self.rewrite_links_selector.push((selector, rewrite_links));
        Ok(self)
    }

    pub(crate) fn build<'a, 'b>(self) -> RewriteStrSettings<'a, 'b> {
        let mut element_content_handlers = vec![];

        for (selector, rewrite_links) in self.rewrite_links_selector.clone() {
            element_content_handlers.push(element!(selector, rewrite_links.build()));
        }

        RewriteStrSettings {
            element_content_handlers,
            ..Default::default()
        }
    }
}
