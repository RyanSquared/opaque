use color_eyre::eyre::{eyre, Result};
use lol_html::{element, RewriteStrSettings};

mod rewrite_links;
use rewrite_links::RewriteLinks;

mod convert_ansi;
use convert_ansi::ConvertAnsi;

type Closure = Box<
    dyn FnMut(
        &mut lol_html::html_content::Element,
    ) -> std::result::Result<(), Box<(dyn std::error::Error + Send + Sync)>>,
>;

trait PostProcessor {
    fn build(self) -> Closure;
}

#[derive(Default)]
pub(crate) struct PostProcessingBuilder {
    rewrite_links_selector: Vec<(String, RewriteLinks)>,
    convert_ansi_selector: Vec<(String, ConvertAnsi)>,
}

impl PostProcessingBuilder {
    pub(crate) fn rewrite_links(
        mut self,
        selector: String,
        url: String,
        attribute: Option<String>,
    ) -> Result<Self> {
        let attribute = attribute
            .or_else(|| {
                selector
                    .split('[')
                    .nth(1)
                    .map(|v| String::from(v.trim_end_matches(']')))
            })
            .ok_or(eyre!("rewrite_links: an attribute could not be derived from selector"))?;
        let rewrite_links = RewriteLinks::new(url, attribute);
        self.rewrite_links_selector.push((selector, rewrite_links));
        Ok(self)
    }

    pub(crate) fn convert_ansi(
        mut self,
        selector: String,
        source_file_path: String,
        subdirectory: String,
    ) -> Result<Self> {
        self.convert_ansi_selector
            .push((selector, ConvertAnsi::new(source_file_path, subdirectory)?));
        Ok(self)
    }

    pub(crate) fn build<'a, 'b>(self) -> RewriteStrSettings<'a, 'b> {
        let mut element_content_handlers = vec![];

        for (selector, rewrite_links) in self.rewrite_links_selector.clone() {
            element_content_handlers.push(element!(selector, rewrite_links.build()));
        }

        for (selector, convert_ansi) in self.convert_ansi_selector {
            element_content_handlers.push(element!(selector, convert_ansi.build()));
        }

        RewriteStrSettings {
            element_content_handlers,
            ..Default::default()
        }
    }
}
