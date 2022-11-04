use std::path::PathBuf;
use anyhow::{Context, Result};
use tracing::{debug, span, Level};

use opaque_ansi::rewrite_ansi_to_html;

#[derive(Debug, Clone)]
pub(crate) struct ConvertAnsi {
    source_directory: PathBuf,
}

impl ConvertAnsi {
    pub(crate) fn new(source_file_path: String) -> Result<Self> {
        let source_directory = PathBuf::from(source_file_path);
        source_directory
            .try_exists()
            .context(format!("{source_directory:?} does not exist"))?;
        Ok(ConvertAnsi { source_directory })
    }
}

impl super::PostProcessor for ConvertAnsi {
    fn build(self) -> super::Closure {
        Box::new(move |el| {
            let _span = span!(target: "convert_ansi", Level::INFO, "convert_ansi").entered();
            // Try to load the file, sync because we're not in an async context
            let Some(filename) = el.get_attribute("source") else {
                return Ok(());
            };
            let path = self.source_directory.clone().join(filename.as_str().trim_matches('/'));
            debug!(?path, "loading ANSI output file");
            // Note: a leading slash *replaces* the PathBuf, this MUST NOT happen
            let file = std::fs::File::open(path)?;
            let file_content = std::io::read_to_string(file)?;
            debug!("formatting file");
            let html_output = rewrite_ansi_to_html(file_content.as_str());
            el.replace(
                html_output.as_str(),
                lol_html::html_content::ContentType::Html,
            );
            Ok(())
        })
    }
}
