use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub is_directory: bool,
    pub created_at: chrono::DateTime<chrono::Local>,
}

impl FileEntry {
    pub fn new(path: PathBuf, tags: Vec<String>) -> Self {
        Self {
            is_directory: path.is_dir(),
            path,
            tags,
            created_at: chrono::Local::now(),
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        let path_matches = self
            .path
            .to_string_lossy()
            .to_lowercase()
            .contains(&query_lower);
        let tag_matches = self
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query_lower));

        path_matches || tag_matches
    }
}
