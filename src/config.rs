use crate::file_entry::FileEntry;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config_path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("file_manager_config.json"),
        }
    }

    pub fn save_entries(&self, entries: &[FileEntry]) -> Result<(), String> {
        match serde_json::to_string_pretty(entries) {
            Ok(json) => {
                fs::write(&self.config_path, json).map_err(|e| format!("保存配置失败: {}", e))
            }
            Err(e) => Err(format!("序列化失败: {}", e)),
        }
    }

    pub fn load_entries(&self) -> Vec<FileEntry> {
        match fs::read_to_string(&self.config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
