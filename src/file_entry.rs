use pinyin::ToPinyin;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

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
    pub child_entries: Vec<String>, // 集合类型的子项目ID
    // 保持向后兼容性
    #[serde(default)]
    pub is_directory: bool,
    // 新增唯一ID字段
    #[serde(default = "generate_id")]
    pub id: String,
    // 向后兼容的旧格式索引
    #[serde(default)]
    pub legacy_child_entries: Vec<usize>,
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
            id: generate_id(),
            legacy_child_entries: Vec::new(),
        }
    }

    /// 从旧版本数据迁移时使用的构造函数
    pub fn migrate_from_old(mut self) -> Self {
        // 如果entry_type是默认值，根据is_directory重新设置
        if self.entry_type == EntryType::File && self.is_directory {
            self.entry_type = EntryType::Directory;
        }

        // 如果没有ID，生成一个新的
        if self.id.is_empty() {
            self.id = generate_id();
        }

        // 保存旧的索引引用以便后续迁移
        if !self.child_entries.is_empty() && self.legacy_child_entries.is_empty() {
            // 如果child_entries包含的是数字字符串，说明是从索引转换来的
            let mut legacy_indices = Vec::new();
            for child_id in &self.child_entries {
                if let Ok(index) = child_id.parse::<usize>() {
                    legacy_indices.push(index);
                }
            }
            if !legacy_indices.is_empty() {
                self.legacy_child_entries = legacy_indices;
                self.child_entries.clear(); // 清空，等待ID迁移
            }
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
            id: generate_id(),
            legacy_child_entries: Vec::new(),
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
            id: generate_id(),
            legacy_child_entries: Vec::new(),
        }
    }

    /// 创建集合条目
    pub fn new_collection(
        name: String,
        nickname: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        child_entry_ids: Vec<String>,
    ) -> Self {
        Self {
            path: PathBuf::from(format!("collection://{}", name)), // 虚拟路径
            name,
            nickname,
            description,
            tags,
            entry_type: EntryType::Collection,
            url: None,
            child_entries: child_entry_ids,
            is_directory: false,
            id: generate_id(),
            legacy_child_entries: Vec::new(),
        }
    }

    /// 添加子项目到集合（使用ID）
    #[allow(dead_code)]
    pub fn add_child_entry(&mut self, entry_id: &str) {
        if self.entry_type == EntryType::Collection
            && !self.child_entries.contains(&entry_id.to_string())
        {
            self.child_entries.push(entry_id.to_string());
        }
    }

    /// 从集合中移除子项目（使用ID）
    #[allow(dead_code)]
    pub fn remove_child_entry(&mut self, entry_id: &str) {
        if self.entry_type == EntryType::Collection {
            self.child_entries.retain(|x| x != entry_id);
        }
    }

    /// 获取子项目ID列表
    #[allow(dead_code)]
    pub fn get_child_entries(&self) -> &Vec<String> {
        &self.child_entries
    }

    /// 检查是否有需要迁移的旧索引数据
    pub fn has_legacy_child_entries(&self) -> bool {
        !self.legacy_child_entries.is_empty()
    }

    /// 获取需要迁移的旧索引数据
    pub fn get_legacy_child_entries(&self) -> &Vec<usize> {
        &self.legacy_child_entries
    }

    /// 清除已迁移的旧索引数据
    pub fn clear_legacy_child_entries(&mut self) {
        self.legacy_child_entries.clear();
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
        if self.name.to_lowercase().contains(&query_lower) || self.matches_pinyin(&self.name, query)
        {
            return true;
        }

        // 搜索昵称（包括拼音）
        if let Some(nickname) = &self.nickname {
            if nickname.to_lowercase().contains(&query_lower)
                || self.matches_pinyin(nickname, query)
            {
                return true;
            }
        }

        // 搜索标签
        if self
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query_lower))
        {
            return true;
        }

        // 搜索描述（包括拼音）
        if let Some(description) = &self.description {
            if description.to_lowercase().contains(&query_lower)
                || self.matches_pinyin(description, query)
            {
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

/// 生成唯一ID
fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_entry_id_generation() {
        let entry = FileEntry::new(
            PathBuf::from("/test/path"),
            "Test Entry".to_string(),
            Some("Test description".to_string()),
            vec!["#tag1".to_string(), "#tag2".to_string()],
            false,
        );

        // 确保每个条目都有唯一的ID
        assert!(!entry.id.is_empty());
        assert_ne!(entry.id, "");

        // 创建另一个条目，确保ID不同
        let entry2 = FileEntry::new(
            PathBuf::from("/test/path2"),
            "Test Entry 2".to_string(),
            None,
            vec![],
            false,
        );

        assert_ne!(entry.id, entry2.id);
    }

    #[test]
    fn test_collection_with_id_system() {
        // 创建一些测试条目
        let file1 = FileEntry::new(
            PathBuf::from("/test/file1.txt"),
            "File 1".to_string(),
            None,
            vec![],
            false,
        );

        let file2 = FileEntry::new(
            PathBuf::from("/test/file2.txt"),
            "File 2".to_string(),
            None,
            vec![],
            false,
        );

        // 创建包含这些文件的集合
        let child_ids = vec![file1.id.clone(), file2.id.clone()];
        let collection = FileEntry::new_collection(
            "Test Collection".to_string(),
            None,
            Some("A test collection".to_string()),
            vec!["#collection".to_string()],
            child_ids.clone(),
        );

        // 验证集合正确存储了子项目的ID
        assert_eq!(collection.child_entries.len(), 2);
        assert!(collection.child_entries.contains(&file1.id));
        assert!(collection.child_entries.contains(&file2.id));
        assert_eq!(collection.entry_type, EntryType::Collection);
    }

    #[test]
    fn test_legacy_migration() {
        // 模拟旧格式的数据（使用索引）
        let mut old_collection = FileEntry {
            path: PathBuf::from("collection://Test"),
            name: "Test Collection".to_string(),
            nickname: None,
            description: None,
            tags: vec![],
            entry_type: EntryType::Collection,
            url: None,
            child_entries: vec!["0".to_string(), "1".to_string()], // 模拟从索引转换的字符串
            is_directory: false,
            id: "".to_string(), // 旧数据没有ID
            legacy_child_entries: vec![],
        };

        // 执行迁移
        old_collection = old_collection.migrate_from_old();

        // 验证迁移结果
        assert!(!old_collection.id.is_empty()); // 应该生成了新的ID
        assert!(old_collection.has_legacy_child_entries()); // 应该保存了旧的索引数据
        assert_eq!(old_collection.get_legacy_child_entries(), &vec![0, 1]);
        assert!(old_collection.child_entries.is_empty()); // 子项目列表应该被清空等待ID迁移
    }

    #[test]
    fn test_add_remove_child_entries_by_id() {
        let mut collection =
            FileEntry::new_collection("Test Collection".to_string(), None, None, vec![], vec![]);

        let test_id = "test-child-id-123".to_string();

        // 添加子项目
        collection.add_child_entry(&test_id);
        assert!(collection.child_entries.contains(&test_id));
        assert_eq!(collection.child_entries.len(), 1);

        // 重复添加应该被忽略
        collection.add_child_entry(&test_id);
        assert_eq!(collection.child_entries.len(), 1);

        // 移除子项目
        collection.remove_child_entry(&test_id);
        assert!(!collection.child_entries.contains(&test_id));
        assert_eq!(collection.child_entries.len(), 0);
    }

    #[test]
    fn test_id_system_prevents_reference_errors() {
        // 模拟删除操作后的场景
        let mut entries = vec![
            FileEntry::new(
                PathBuf::from("/file1"),
                "File 1".to_string(),
                None,
                vec![],
                false,
            ),
            FileEntry::new(
                PathBuf::from("/file2"),
                "File 2".to_string(),
                None,
                vec![],
                false,
            ),
            FileEntry::new(
                PathBuf::from("/file3"),
                "File 3".to_string(),
                None,
                vec![],
                false,
            ),
        ];

        // 创建一个包含所有文件的集合
        let child_ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
        let collection = FileEntry::new_collection(
            "All Files".to_string(),
            None,
            None,
            vec![],
            child_ids.clone(),
        );
        entries.push(collection);

        // 验证集合包含正确的文件ID
        let collection_ref = &entries[3];
        assert_eq!(collection_ref.child_entries.len(), 3);
        for entry in entries[0..3].iter() {
            assert!(collection_ref.child_entries.contains(&entry.id));
        }

        // 模拟删除中间的文件（索引1）
        let removed_id = entries[1].id.clone();
        let remaining_file1_id = entries[0].id.clone();
        let remaining_file3_id = entries[2].id.clone();
        entries.remove(1); // 删除 "File 2"

        // 更新集合以移除已删除的文件ID
        let collection_mut = &mut entries[2]; // 注意索引变化：原来是[3]，现在是[2]
        collection_mut.child_entries.retain(|id| id != &removed_id);

        // 验证：
        // 1. 集合现在只包含2个子项目
        assert_eq!(collection_mut.child_entries.len(), 2);
        // 2. 集合不再包含已删除文件的ID
        assert!(!collection_mut.child_entries.contains(&removed_id));
        // 3. 集合仍然包含其他文件的ID
        assert!(collection_mut.child_entries.contains(&remaining_file1_id)); // "File 1"
        assert!(collection_mut.child_entries.contains(&remaining_file3_id)); // "File 3"

        // 4. 通过ID可以正确找到对应的文件
        let child_entries_clone = collection_mut.child_entries.clone();
        for child_id in &child_entries_clone {
            let found = entries.iter().find(|e| &e.id == child_id);
            assert!(found.is_some(), "应该能通过ID找到对应的文件");
        }
    }

    #[test]
    fn test_web_link_entry_has_id() {
        let web_entry = FileEntry::new_web_link(
            "Test Website".to_string(),
            "https://example.com".to_string(),
            Some("Example".to_string()),
            Some("A test website".to_string()),
            vec!["#web".to_string()],
        );

        assert!(!web_entry.id.is_empty());
        assert_eq!(web_entry.entry_type, EntryType::WebLink);
        assert_eq!(web_entry.url, Some("https://example.com".to_string()));
    }
}
