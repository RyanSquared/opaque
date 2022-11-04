use tracing::{debug, span, Level};

#[derive(Debug, Clone)]
pub(crate) struct RewriteLinks {
    url: String,
    attribute: String,
}

impl RewriteLinks {
    pub(crate) fn new(url: String, attribute: String) -> Self {
        RewriteLinks { url, attribute }
    }
}

impl super::PostProcessor for RewriteLinks {
    fn build(self) -> super::Closure {
        Box::new(move |el| {
            let _span = span!(target: "rewrite_links", Level::INFO, "rewrite_links").entered();
            if let Some(src) = el.get_attribute(self.attribute.as_str()) {
                let url = &self.url;
                let rewritten_url = format!("{url}{src}");
                debug!("rewriting from: {url}, to: {rewritten_url}");
                let result = el.set_attribute(self.attribute.as_str(), rewritten_url.as_str());
                result.map_err(|e| Box::new(e) as Box<(dyn std::error::Error + Send + Sync)>)
            } else {
                Ok(())
            }
        })
    }
}
