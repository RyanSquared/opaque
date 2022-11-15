use color_eyre::eyre::Result;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::path::PathBuf;
use tracing::{debug, span, Level};

use opaque_ansi::rewrite_ansi_to_html;

static CACHE: OnceCell<Mutex<uluru::LRUCache<(String, String), 256>>> = OnceCell::new();

#[derive(Debug, Clone)]
pub(crate) struct ConvertAnsi {
    source_directory: PathBuf,
    subdirectory: PathBuf,
}

impl ConvertAnsi {
    pub(crate) fn new(source_file_path: String, subdirectory: String) -> Result<Self> {
        if CACHE.get().is_none() {
            CACHE
                .set(Mutex::new(uluru::LRUCache::default()))
                .expect("unset cache can't be initialized");
        }
        let source_directory = PathBuf::from(source_file_path);
        source_directory.try_exists()?;
        Ok(ConvertAnsi { source_directory, subdirectory: PathBuf::from(subdirectory) })
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

            let mut path = self.source_directory.clone();
            if el.get_attribute("relative").is_some() {
                path = path.join(self.subdirectory.as_path())
            }
            path = path.join(filename.as_str().trim_matches('/'));

            debug!(?path, "loading ANSI output file");

            // Return auto generated output from the cache if available
            if let Some(cache_mutex) = CACHE.get() {
                let mut cache = cache_mutex.lock();
                if let Some((_, hit)) = cache.find(|(k, _)| filename == *k) {
                    debug!(?filename, "cache hit");
                    el.replace(
                        hit.as_str(),
                        lol_html::html_content::ContentType::Html,
                    );
                    return Ok(())
                }
            }

            // Note: a leading slash *replaces* the PathBuf, this MUST NOT happen
            let file = std::fs::File::open(path)?;
            let file_content = std::io::read_to_string(file)?;
            debug!("formatting file");
            let html_output = rewrite_ansi_to_html(file_content.as_str());
            el.replace(
                html_output.as_str(),
                lol_html::html_content::ContentType::Html,
            );

            if let Some(cache_mutex) = CACHE.get() {
                let mut cache = cache_mutex.lock();
                if cache.find(|(k, _)| filename == *k).is_none() {
                    debug!(?filename, "cache miss, updating");
                    cache.insert((filename, html_output));
                }
            }

            Ok(())
        })
    }
}
