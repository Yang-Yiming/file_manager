use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use pinyin::ToPinyin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    WebLink,
    Collection,
}

impl Default for EntryType {
    fn default() -> Self {
        EntryType::File
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub nickname: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub entry_type: EntryType,
    pub url: Option<String>, // 网页链接地址
    #[serde(default)]
    pub child_entries: Vec<usize>, // 集合类型的子项目索引
    // 保持向后兼容性
    #[serde(default)]
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
        let entry_type = if is_directory {
            EntryType::Directory
        } else {
            EntryType::File
        };
        
        Self {
            path,
            name,
            nickname: None,
            description,
            tags,
            entry_type,
            url: None,
            child_entries: Vec::new(),
            is_directory,
        }
    }

    /// 从旧版本数据迁移时使用的构造函数
    pub fn migrate_from_old(mut self) -> Self {
        // 如果entry_type是默认值，根据is_directory重新设置
        if self.entry_type == EntryType::File && self.is_directory {
            self.entry_type = EntryType::Directory;
        }
        self
    }

    pub fn new_with_nickname(
        path: PathBuf,
        name: String,
        nickname: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        is_directory: bool,
    ) -> Self {
        let entry_type = if is_directory {
            EntryType::Directory
        } else {
            EntryType::File
        };
        
        Self {
            path,
            name,
            nickname,
            description,
            tags,
            entry_type,
            url: None,
            child_entries: Vec::new(),
            is_directory,
        }
    }

    /// 创建网页链接条目
    pub fn new_web_link(
        name: String,
        url: String,
        nickname: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            path: PathBuf::from(&url), // 将URL作为路径存储，用于显示
            name,
            nickname,
            description,
            tags,
            entry_type: EntryType::WebLink,
            url: Some(url),
            child_entries: Vec::new(),
            is_directory: false,
        }
    }

    /// 创建集合条目
    pub fn new_collection(
        name: String,
        nickname: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        child_entries: Vec<usize>,
    ) -> Self {
        Self {
            path: PathBuf::from(format!("collection://{}", name)), // 虚拟路径
            name,
            nickname,
            description,
            tags,
            entry_type: EntryType::Collection,
            url: None,
            child_entries,
            is_directory: false,
        }
    }

    /// 添加子项目到集合
    #[allow(dead_code)]
    pub fn add_child_entry(&mut self, entry_index: usize) {
        if self.entry_type == EntryType::Collection && !self.child_entries.contains(&entry_index) {
            self.child_entries.push(entry_index);
        }
    }

    /// 从集合中移除子项目
    #[allow(dead_code)]
    pub fn remove_child_entry(&mut self, entry_index: usize) {
        if self.entry_type == EntryType::Collection {
            self.child_entries.retain(|&x| x != entry_index);
        }
    }

    /// 获取子项目索引列表
    #[allow(dead_code)]
    pub fn get_child_entries(&self) -> &Vec<usize> {
        &self.child_entries
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