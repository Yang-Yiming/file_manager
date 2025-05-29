use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_directory: bool,
    pub created_at: chrono::DateTime<chrono::Local>,
}

impl FileEntry {

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
        let name_matches = self
            .name
            .to_lowercase()
            .contains(&query_lower);
        let description_matches = self
            .description
            .as_ref()
            .map(|desc| desc.to_lowercase().contains(&query_lower))
            .unwrap_or(false);
        let tag_matches = self
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query_lower));

        path_matches || name_matches || description_matches || tag_matches
    }
}
