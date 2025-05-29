use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use pinyin::ToPinyin;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub nickname: Option<String>,
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
            nickname: None,
            description,
            tags,
            is_directory,
        }
    }

    pub fn new_with_nickname(
        path: PathBuf,
        name: String,
        nickname: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        is_directory: bool,
    ) -> Self {
        Self {
            path,
            name,
            nickname,
            description,
            tags,
            is_directory,
        }
    }

    /// 将中文转换为拼音首字母
    fn to_pinyin_initials(text: &str) -> String {
        text.to_pinyin()
            .map(|pinyin| {
                pinyin
                    .map(|p| p.first_letter().to_uppercase().to_string())
                    .unwrap_or_else(|| "".to_string())
            })
            .collect::<String>()
    }

    /// 将中文转换为完整拼音
    fn to_full_pinyin(text: &str) -> String {
        text.to_pinyin()
            .map(|pinyin| {
                pinyin
                    .map(|p| p.plain().to_string())
                    .unwrap_or_else(|| "".to_string())
            })
            .collect::<String>()
    }

    /// 检查文本是否匹配拼音搜索
    fn matches_pinyin(&self, text: &str, query: &str) -> bool {
        if query.is_empty() || text.is_empty() {
            return false;
        }

        let query_lower = query.to_lowercase();
        
        // 检查拼音首字母匹配
        let pinyin_initials = Self::to_pinyin_initials(text).to_lowercase();
        if pinyin_initials.contains(&query_lower) {
            return true;
        }

        // 检查完整拼音匹配
        let full_pinyin = Self::to_full_pinyin(text).to_lowercase();
        if full_pinyin.contains(&query_lower) {
            return true;
        }

        false
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        
        // 搜索名称（包括拼音）
        if self.name.to_lowercase().contains(&query_lower) 
            || self.matches_pinyin(&self.name, query) {
            return true;
        }

        // 搜索昵称（包括拼音）
        if let Some(nickname) = &self.nickname {
            if nickname.to_lowercase().contains(&query_lower) 
                || self.matches_pinyin(nickname, query) {
                return true;
            }
        }

        // 搜索标签
        if self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
            return true;
        }

        // 搜索描述（包括拼音）
        if let Some(description) = &self.description {
            if description.to_lowercase().contains(&query_lower) 
                || self.matches_pinyin(description, query) {
                return true;
            }
        }

        false
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