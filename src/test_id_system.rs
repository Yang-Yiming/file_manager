#[cfg(test)]
mod tests {
    use crate::file_entry::{FileEntry, EntryType};
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
        let mut collection = FileEntry::new_collection(
            "Test Collection".to_string(),
            None,
            None,
            vec![],
            vec![],
        );
        
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
            FileEntry::new(PathBuf::from("/file1"), "File 1".to_string(), None, vec![], false),
            FileEntry::new(PathBuf::from("/file2"), "File 2".to_string(), None, vec![], false),
            FileEntry::new(PathBuf::from("/file3"), "File 3".to_string(), None, vec![], false),
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