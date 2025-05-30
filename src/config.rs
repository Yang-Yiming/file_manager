use crate::file_entry::FileEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme_mode: String,
    pub data_file_path: Option<String>, // 用户数据文件路径
    pub compact_mode: bool, // 紧凑模式
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: "Light".to_string(),
            data_file_path: None,
            compact_mode: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserData {
    pub entries: Vec<FileEntry>,
    pub version: String,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            version: "0.2.0".to_string(),
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

    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
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

    pub fn load_config(&self) -> Result<AppConfig, String> {
        match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                serde_json::from_str::<AppConfig>(&content)
                    .map_err(|e: serde_json::Error| format!("解析配置失败: {}", e))
            }
            Err(_) => Ok(AppConfig::default()),
        }
    }
}

pub struct DataManager {
    data_path: PathBuf,
}

impl Default for DataManager {
    fn default() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            data_path: exe_dir.join("file_manager_data.json"),
        }
    }
}

impl DataManager {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            data_path: exe_dir.join("file_manager_data.json"),
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        Self {
            data_path: path,
        }
    }

    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }

    pub fn save_data(&self, data: &UserData) -> Result<(), String> {
        // 确保目录存在
        if let Some(parent) = self.data_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        match serde_json::to_string_pretty(data) {
            Ok(json) => {
                fs::write(&self.data_path, json).map_err(|e| format!("保存数据失败: {}", e))
            }
            Err(e) => Err(format!("序列化失败: {}", e)),
        }
    }

    pub fn load_data(&self) -> Result<UserData, String> {
        match fs::read_to_string(&self.data_path) {
            Ok(content) => {
                // 尝试新格式
                serde_json::from_str::<UserData>(&content)
                    .or_else(|_| {
                        // 兼容旧格式：直接是entries数组或者包含entries的Config
                        if let Ok(entries) = serde_json::from_str::<Vec<FileEntry>>(&content) {
                            Ok(UserData {
                                entries,
                                version: "0.2.0".to_string(),
                            })
                        } else if let Ok(old_config) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(entries_value) = old_config.get("entries") {
                                let entries: Vec<FileEntry> = serde_json::from_value(entries_value.clone())
                                    .unwrap_or_default();
                                Ok(UserData {
                                    entries,
                                    version: "0.2.0".to_string(),
                                })
                            } else {
                                Ok(UserData::default())
                            }
                        } else {
                            Ok(UserData::default())
                        }
                    })
                    .map_err(|e: serde_json::Error| format!("解析数据失败: {}", e))
            }
            Err(_) => Ok(UserData::default()),
        }
    }
}