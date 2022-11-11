use std::path::Path;

use color_eyre::eyre::Result;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tracing::debug;
use walkdir::WalkDir;

use crate::state::{FrontMatter, Page, PageMap};

#[tracing::instrument]
pub(crate) async fn walk_directory(path: impl AsRef<Path> + std::fmt::Debug) -> Result<PageMap> {
    let mut page_map = PageMap::new();
    'walkdir: for entry in WalkDir::new(path).follow_links(true) {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            debug!(?entry, "loading file front matter");
            let file = File::open(entry.path()).await?;
            let reader = BufReader::new(file);
            let mut lines = vec![];
            let mut line_reader = reader.lines();
            if let Some(line) = line_reader.next_line().await? {
                if line.as_str() != "---" {
                    continue;
                }
            }
            while let Some(line) = line_reader.next_line().await? {
                if line.as_str() == "---" {
                    let front_matter: FrontMatter =
                        serde_yaml::from_str(lines.join("\n").as_str())?;
                    let slug = front_matter.slug();
                    debug!(
                        ?entry,
                        ?slug,
                        "storing parsed front_matter and file_path under slug"
                    );
                    page_map.insert(
                        slug,
                        Page {
                            front_matter,
                            file_path: entry.path().to_path_buf(),
                        },
                    );
                    continue 'walkdir;
                }
                lines.push(line);
            }
        }
    }
    Ok(page_map)
}
