use crate::file_entry::FileEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub enable_chinese_font: bool,
    pub entries: Vec<FileEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enable_chinese_font: false,
            entries: Vec::new(),
        }
    }
}

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

    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        match serde_json::to_string_pretty(config) {
            Ok(json) => {
                fs::write(&self.config_path, json).map_err(|e| format!("保存配置失败: {}", e))
            }
            Err(e) => Err(format!("序列化失败: {}", e)),
        }
    }

    pub fn load_config(&self) -> AppConfig {
        match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                // 尝试加载新格式配置
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    config
                } else {
                    // 兼容旧格式：只有entries数组
                    let entries: Vec<FileEntry> = serde_json::from_str(&content).unwrap_or_default();
                    AppConfig {
                        enable_chinese_font: false,
                        entries,
                    }
                }
            }
            Err(_) => AppConfig::default(),
        }
    }



    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
