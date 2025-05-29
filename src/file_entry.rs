use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_directory: bool,
}

impl FileEntry {
    pub fn new(
        path: PathBuf,
        name: String,
        description: Option<String>,
        tags: Vec<String>,
        is_directory: bool,
    ) -> Self {
        Self {
            path,
            name,
            description,
            tags,
            is_directory,
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        
        // 搜索名称、标签和描述
        self.name.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            || self.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
    }

    /// 解析标签字符串，只支持 # 标签
    pub fn parse_tags(tag_input: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // 按空格和逗号分割
        for part in tag_input.split_whitespace() {
            for tag in part.split(',') {
                let tag = tag.trim();
                if tag.is_empty() {
                    continue;
                }
                
                // 处理 # 标签
                if tag.starts_with('#') {
                    let tag_name = tag.strip_prefix('#').unwrap_or(tag);
                    if !tag_name.is_empty() {
                        tags.push(format!("#{}", tag_name));
                    }
                }
                // 普通标签，自动添加 # 前缀
                else {
                    tags.push(format!("#{}", tag));
                }
            }
        }
        
        // 去重并排序
        tags.sort();
        tags.dedup();
        tags
    }

    /// 获取所有标签（只返回hash标签）
    pub fn get_tag_categories(&self) -> (Vec<String>, Vec<String>) {
        let mut hash_tags = Vec::new();
        let path_tags = Vec::new(); // 空的路径标签
        
        for tag in &self.tags {
            if tag.starts_with('#') {
                hash_tags.push(tag.clone());
            }
        }
        
        (hash_tags, path_tags)
    }
}