use crate::file_entry::FileEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub entries: Vec<FileEntry>,
    pub config_path: Option<String>, // 用户指定的配置文件路径
}

impl Default for Config {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            config_path: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            config_path: exe_dir.join("file_manager_config.json"),
        }
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            config_path: exe_dir.join("file_manager_config.json"),
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
        }
    }

    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = path;
    }

    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn save_config(&self, config: &Config) -> Result<(), String> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        match serde_json::to_string_pretty(config) {
            Ok(json) => {
                fs::write(&self.config_path, json).map_err(|e| format!("保存配置失败: {}", e))
            }
            Err(e) => Err(format!("序列化失败: {}", e)),
        }
    }

    pub fn load_config(&self) -> Result<Config, String> {
        match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                serde_json::from_str::<Config>(&content)
                    .or_else(|_| {
                        // 兼容旧格式：只有entries数组
                        let entries: Vec<FileEntry> = serde_json::from_str(&content).unwrap_or_default();
                        Ok(Config { 
                            entries,
                            config_path: None,
                        })
                    })
                    .map_err(|e: serde_json::Error| format!("解析配置失败: {}", e))
            }
            Err(_) => Ok(Config::default()),
        }
    }
}