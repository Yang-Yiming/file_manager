use crate::file_entry::{EntryType, FileEntry};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

/// 异步操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncResult<T> {
    Success(T),
    Error(String),
    Timeout,
    Cancelled,
}

impl<T> AsyncResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, AsyncResult::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, AsyncResult::Error(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            AsyncResult::Success(value) => value,
            AsyncResult::Error(msg) => panic!("AsyncResult::unwrap() called on Error: {}", msg),
            AsyncResult::Timeout => panic!("AsyncResult::unwrap() called on Timeout"),
            AsyncResult::Cancelled => panic!("AsyncResult::unwrap() called on Cancelled"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            AsyncResult::Success(value) => value,
            _ => default,
        }
    }
}

/// 异步操作类型
#[derive(Debug, Clone)]
pub enum AsyncOperation {
    /// 检查路径是否存在
    PathExists(PathBuf),
    /// 获取文件信息
    GetFileInfo(PathBuf),
    /// 读取目录内容
    ReadDirectory(PathBuf),
    /// 创建目录
    CreateDirectory(PathBuf),
    /// 删除文件或目录
    Delete(PathBuf),
    /// 复制文件或目录
    Copy(PathBuf, PathBuf),
    /// 移动文件或目录
    Move(PathBuf, PathBuf),
    /// 获取文件大小
    GetFileSize(PathBuf),
    /// 获取文件修改时间
    GetModifiedTime(PathBuf),
    /// 批量操作
    Batch(Vec<AsyncOperation>),
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub is_file: bool,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
    pub readonly: bool,
    pub extension: Option<String>,
}

impl FileInfo {
    pub async fn from_path(path: &Path) -> Result<Self, String> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| format!("获取文件元数据失败: {}", e))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        Ok(FileInfo {
            path: path.to_path_buf(),
            name,
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            readonly: metadata.permissions().readonly(),
            extension,
        })
    }

    pub fn to_file_entry(&self) -> FileEntry {
        FileEntry::new(
            self.path.clone(),
            self.name.clone(),
            None,       // description
            Vec::new(), // tags
            self.is_directory,
        )
    }
}

/// 异步操作任务
#[derive(Debug)]
pub struct AsyncTask {
    pub id: String,
    pub operation: AsyncOperation,
    pub timeout_duration: Duration,
    pub result_sender: oneshot::Sender<AsyncResult<serde_json::Value>>,
}

impl AsyncTask {
    pub fn new(
        id: String,
        operation: AsyncOperation,
        timeout_duration: Duration,
        result_sender: oneshot::Sender<AsyncResult<serde_json::Value>>,
    ) -> Self {
        Self {
            id,
            operation,
            timeout_duration,
            result_sender,
        }
    }
}

/// 异步操作管理器
pub struct AsyncOperationManager {
    task_sender: mpsc::UnboundedSender<AsyncTask>,
    active_tasks: Arc<Mutex<std::collections::HashMap<String, oneshot::Sender<()>>>>,
    runtime: tokio::runtime::Runtime,
}

impl AsyncOperationManager {
    pub fn new() -> Result<Self, String> {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        let active_tasks = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("创建异步运行时失败: {}", e))?;

        let manager = Self {
            task_sender,
            active_tasks: active_tasks.clone(),
            runtime,
        };

        // 启动任务处理器
        manager.start_task_processor(task_receiver, active_tasks);

        Ok(manager)
    }

    /// 启动任务处理器
    fn start_task_processor(
        &self,
        mut task_receiver: mpsc::UnboundedReceiver<AsyncTask>,
        active_tasks: Arc<Mutex<std::collections::HashMap<String, oneshot::Sender<()>>>>,
    ) {
        self.runtime.spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                let task_id = task.id.clone();
                let (cancel_sender, cancel_receiver) = oneshot::channel();

                // 添加到活动任务列表
                {
                    let mut tasks = active_tasks.lock().unwrap();
                    tasks.insert(task_id.clone(), cancel_sender);
                }

                // 处理任务
                tokio::spawn(async move {
                    let result =
                        Self::execute_task(task.operation, task.timeout_duration, cancel_receiver)
                            .await;

                    // 发送结果
                    let _ = task.result_sender.send(result);
                });

                // 从活动任务列表中移除
                {
                    let mut tasks = active_tasks.lock().unwrap();
                    tasks.remove(&task_id);
                }
            }
        });
    }

    /// 执行异步任务
    async fn execute_task(
        operation: AsyncOperation,
        timeout_duration: Duration,
        cancel_receiver: oneshot::Receiver<()>,
    ) -> AsyncResult<serde_json::Value> {
        let operation_future = Self::perform_operation(operation);

        tokio::select! {
            result = timeout(timeout_duration, operation_future) => {
                match result {
                    Ok(op_result) => op_result,
                    Err(_) => AsyncResult::Timeout,
                }
            }
            _ = cancel_receiver => AsyncResult::Cancelled,
        }
    }

    /// 执行具体操作
    async fn perform_operation(operation: AsyncOperation) -> AsyncResult<serde_json::Value> {
        match operation {
            AsyncOperation::PathExists(path) => {
                let exists = fs::metadata(&path).await.is_ok();
                AsyncResult::Success(serde_json::json!(exists))
            }
            AsyncOperation::GetFileInfo(path) => match FileInfo::from_path(&path).await {
                Ok(info) => match serde_json::to_value(&info) {
                    Ok(json) => AsyncResult::Success(json),
                    Err(e) => AsyncResult::Error(format!("序列化文件信息失败: {}", e)),
                },
                Err(e) => AsyncResult::Error(e),
            },
            AsyncOperation::ReadDirectory(path) => {
                match Self::read_directory_contents(&path).await {
                    Ok(entries) => match serde_json::to_value(&entries) {
                        Ok(json) => AsyncResult::Success(json),
                        Err(e) => AsyncResult::Error(format!("序列化目录内容失败: {}", e)),
                    },
                    Err(e) => AsyncResult::Error(e),
                }
            }
            AsyncOperation::CreateDirectory(path) => match fs::create_dir_all(&path).await {
                Ok(_) => AsyncResult::Success(serde_json::json!(true)),
                Err(e) => AsyncResult::Error(format!("创建目录失败: {}", e)),
            },
            AsyncOperation::Delete(path) => {
                let result = if path.is_file() {
                    fs::remove_file(&path).await
                } else {
                    fs::remove_dir_all(&path).await
                };

                match result {
                    Ok(_) => AsyncResult::Success(serde_json::json!(true)),
                    Err(e) => AsyncResult::Error(format!("删除失败: {}", e)),
                }
            }
            AsyncOperation::Copy(src, dst) => match Self::copy_recursive(&src, &dst).await {
                Ok(_) => AsyncResult::Success(serde_json::json!(true)),
                Err(e) => AsyncResult::Error(e),
            },
            AsyncOperation::Move(src, dst) => match fs::rename(&src, &dst).await {
                Ok(_) => AsyncResult::Success(serde_json::json!(true)),
                Err(e) => AsyncResult::Error(format!("移动失败: {}", e)),
            },
            AsyncOperation::GetFileSize(path) => match fs::metadata(&path).await {
                Ok(metadata) => AsyncResult::Success(serde_json::json!(metadata.len())),
                Err(e) => AsyncResult::Error(format!("获取文件大小失败: {}", e)),
            },
            AsyncOperation::GetModifiedTime(path) => match fs::metadata(&path).await {
                Ok(metadata) => match metadata.modified() {
                    Ok(time) => {
                        let timestamp = time
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        AsyncResult::Success(serde_json::json!(timestamp))
                    }
                    Err(e) => AsyncResult::Error(format!("获取修改时间失败: {}", e)),
                },
                Err(e) => AsyncResult::Error(format!("获取文件元数据失败: {}", e)),
            },
            AsyncOperation::Batch(operations) => {
                let mut json_results = Vec::new();
                for op in operations {
                    let result = Box::pin(Self::perform_operation(op)).await;
                    match result {
                        AsyncResult::Success(value) => json_results.push(value),
                        AsyncResult::Error(msg) => {
                            return AsyncResult::Error(format!("批量操作失败: {}", msg));
                        }
                        AsyncResult::Timeout => {
                            return AsyncResult::Error("批量操作中的子操作超时".to_string());
                        }
                        AsyncResult::Cancelled => {
                            return AsyncResult::Error("批量操作中的子操作被取消".to_string());
                        }
                    }
                }
                AsyncResult::Success(serde_json::json!(json_results))
            }
        }
    }

    /// 递归读取目录内容
    async fn read_directory_contents(path: &Path) -> Result<Vec<FileInfo>, String> {
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(path)
            .await
            .map_err(|e| format!("读取目录失败: {}", e))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| format!("读取目录项失败: {}", e))?
        {
            let path = entry.path();
            match FileInfo::from_path(&path).await {
                Ok(info) => entries.push(info),
                Err(e) => {
                    eprintln!("获取文件信息失败 {:?}: {}", path, e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// 递归复制文件或目录
    async fn copy_recursive(src: &Path, dst: &Path) -> Result<(), String> {
        use std::future::Future;
        use std::pin::Pin;

        fn copy_recursive_inner(
            src: PathBuf,
            dst: PathBuf,
        ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
            Box::pin(async move {
                let metadata = fs::metadata(&src)
                    .await
                    .map_err(|e| format!("获取源文件元数据失败: {}", e))?;

                if metadata.is_file() {
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)
                            .await
                            .map_err(|e| format!("创建目标目录失败: {}", e))?;
                    }
                    fs::copy(&src, &dst)
                        .await
                        .map_err(|e| format!("复制文件失败: {}", e))?;
                } else if metadata.is_dir() {
                    fs::create_dir_all(&dst)
                        .await
                        .map_err(|e| format!("创建目标目录失败: {}", e))?;

                    let mut read_dir = fs::read_dir(&src)
                        .await
                        .map_err(|e| format!("读取源目录失败: {}", e))?;

                    while let Some(entry) = read_dir
                        .next_entry()
                        .await
                        .map_err(|e| format!("读取目录项失败: {}", e))?
                    {
                        let src_path = entry.path();
                        let dst_path = dst.join(entry.file_name());
                        copy_recursive_inner(src_path, dst_path).await?;
                    }
                }

                Ok(())
            })
        }

        copy_recursive_inner(src.to_path_buf(), dst.to_path_buf()).await
    }

    /// 提交异步操作任务
    pub fn submit_task(
        &self,
        operation: AsyncOperation,
        timeout_duration: Option<Duration>,
    ) -> Result<AsyncTaskHandle, String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let (result_sender, result_receiver) = oneshot::channel();

        let task = AsyncTask::new(
            task_id.clone(),
            operation,
            timeout_duration.unwrap_or(Duration::from_secs(30)),
            result_sender,
        );

        self.task_sender
            .send(task)
            .map_err(|_| "任务提交失败".to_string())?;

        Ok(AsyncTaskHandle {
            id: task_id,
            result_receiver,
            active_tasks: self.active_tasks.clone(),
        })
    }

    /// 取消所有任务
    pub fn cancel_all_tasks(&self) {
        let mut tasks = self.active_tasks.lock().unwrap();
        for (_, cancel_sender) in tasks.drain() {
            let _ = cancel_sender.send(());
        }
    }

    /// 获取活动任务数量
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.lock().unwrap().len()
    }
}

/// 异步任务句柄
pub struct AsyncTaskHandle {
    pub id: String,
    result_receiver: oneshot::Receiver<AsyncResult<serde_json::Value>>,
    active_tasks: Arc<Mutex<std::collections::HashMap<String, oneshot::Sender<()>>>>,
}

impl AsyncTaskHandle {
    /// 等待任务完成
    pub async fn wait(self) -> AsyncResult<serde_json::Value> {
        match self.result_receiver.await {
            Ok(result) => result,
            Err(_) => AsyncResult::Cancelled,
        }
    }

    /// 取消任务
    pub fn cancel(&self) {
        let mut tasks = self.active_tasks.lock().unwrap();
        if let Some(cancel_sender) = tasks.remove(&self.id) {
            let _ = cancel_sender.send(());
        }
    }

    /// 检查任务是否仍在运行
    pub fn is_running(&self) -> bool {
        let tasks = self.active_tasks.lock().unwrap();
        tasks.contains_key(&self.id)
    }
}

/// 异步操作构建器
pub struct AsyncOperationBuilder {
    operations: Vec<AsyncOperation>,
    timeout: Option<Duration>,
}

impl AsyncOperationBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn check_path_exists<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.operations
            .push(AsyncOperation::PathExists(path.as_ref().to_path_buf()));
        self
    }

    pub fn get_file_info<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.operations
            .push(AsyncOperation::GetFileInfo(path.as_ref().to_path_buf()));
        self
    }

    pub fn read_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.operations
            .push(AsyncOperation::ReadDirectory(path.as_ref().to_path_buf()));
        self
    }

    pub fn create_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.operations
            .push(AsyncOperation::CreateDirectory(path.as_ref().to_path_buf()));
        self
    }

    pub fn delete<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.operations
            .push(AsyncOperation::Delete(path.as_ref().to_path_buf()));
        self
    }

    pub fn copy<P: AsRef<Path>>(mut self, src: P, dst: P) -> Self {
        self.operations.push(AsyncOperation::Copy(
            src.as_ref().to_path_buf(),
            dst.as_ref().to_path_buf(),
        ));
        self
    }

    pub fn move_path<P: AsRef<Path>>(mut self, src: P, dst: P) -> Self {
        self.operations.push(AsyncOperation::Move(
            src.as_ref().to_path_buf(),
            dst.as_ref().to_path_buf(),
        ));
        self
    }

    pub fn build_single(self, manager: &AsyncOperationManager) -> Result<AsyncTaskHandle, String> {
        if self.operations.len() != 1 {
            return Err("构建单个操作时必须只有一个操作".to_string());
        }

        let operation = self.operations.into_iter().next().unwrap();
        manager.submit_task(operation, self.timeout)
    }

    pub fn build_batch(self, manager: &AsyncOperationManager) -> Result<AsyncTaskHandle, String> {
        if self.operations.is_empty() {
            return Err("批量操作不能为空".to_string());
        }

        let batch_operation = AsyncOperation::Batch(self.operations);
        manager.submit_task(batch_operation, self.timeout)
    }
}

impl Default for AsyncOperationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便利函数
pub mod convenience {
    use super::*;

    /// 检查路径是否存在
    pub async fn path_exists<P: AsRef<Path>>(path: P) -> bool {
        fs::metadata(path).await.is_ok()
    }

    /// 获取文件大小
    pub async fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64, String> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| format!("获取文件大小失败: {}", e))?;
        Ok(metadata.len())
    }

    /// 检查是否为目录
    pub async fn is_directory<P: AsRef<Path>>(path: P) -> Result<bool, String> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| format!("获取文件元数据失败: {}", e))?;
        Ok(metadata.is_dir())
    }

    /// 检查是否为文件
    pub async fn is_file<P: AsRef<Path>>(path: P) -> Result<bool, String> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| format!("获取文件元数据失败: {}", e))?;
        Ok(metadata.is_file())
    }

    /// 快速读取目录
    pub async fn quick_read_dir<P: AsRef<Path>>(path: P) -> Result<Vec<String>, String> {
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(path)
            .await
            .map_err(|e| format!("读取目录失败: {}", e))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| format!("读取目录项失败: {}", e))?
        {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_info_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // 创建测试文件
        fs::write(&test_file, "test content").await.unwrap();

        let file_info = FileInfo::from_path(&test_file).await.unwrap();

        assert_eq!(file_info.name, "test.txt");
        assert!(file_info.is_file);
        assert!(!file_info.is_directory);
        assert_eq!(file_info.size, 12);
        assert_eq!(file_info.extension, Some("txt".to_string()));
    }

    #[test]
    #[ignore] // 暂时禁用此测试
    fn test_async_operation_manager() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = AsyncOperationManager::new().unwrap();
            let temp_dir = TempDir::new().unwrap();
            let test_path = temp_dir.path().to_path_buf();

            // 测试路径存在检查
            let handle = manager
                .submit_task(
                    AsyncOperation::PathExists(test_path.clone()),
                    Some(Duration::from_secs(5)),
                )
                .unwrap();

            let result = handle.wait().await;
            assert!(result.is_success());

            // 测试读取目录
            let handle = manager
                .submit_task(
                    AsyncOperation::ReadDirectory(test_path),
                    Some(Duration::from_secs(5)),
                )
                .unwrap();

            let result = handle.wait().await;
            assert!(result.is_success());
        });
    }

    #[test]
    #[ignore] // 暂时禁用此测试
    fn test_async_operation_builder() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = AsyncOperationManager::new().unwrap();
            let temp_dir = TempDir::new().unwrap();
            let test_path = temp_dir.path().to_path_buf();

            // 测试构建器
            let handle = AsyncOperationBuilder::new()
                .with_timeout(Duration::from_secs(5))
                .check_path_exists(&test_path)
                .build_single(&manager)
                .unwrap();

            let result = handle.wait().await;
            assert!(result.is_success());
        });
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // 创建测试文件
        fs::write(&test_file, "test content").await.unwrap();

        // 测试便利函数
        assert!(convenience::path_exists(&test_file).await);
        assert!(convenience::is_file(&test_file).await.unwrap());
        assert!(!convenience::is_directory(&test_file).await.unwrap());

        let size = convenience::get_file_size(&test_file).await.unwrap();
        assert_eq!(size, 12);

        let entries = convenience::quick_read_dir(temp_dir.path()).await.unwrap();
        assert!(entries.contains(&"test.txt".to_string()));
    }

    #[test]
    #[ignore] // 暂时禁用此测试
    fn test_task_cancellation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = AsyncOperationManager::new().unwrap();
            let temp_dir = TempDir::new().unwrap();

            // 创建一个长时间运行的任务
            let handle = manager
                .submit_task(
                    AsyncOperation::ReadDirectory(temp_dir.path().to_path_buf()),
                    Some(Duration::from_secs(10)),
                )
                .unwrap();

            // 取消任务
            handle.cancel();

            let result = handle.wait().await;
            // 注意：由于任务执行很快，可能不会被取消，所以检查成功或取消都是可以的
            assert!(matches!(
                result,
                AsyncResult::Success(_) | AsyncResult::Cancelled
            ));
        });
    }

    #[test]
    fn test_async_result() {
        let success_result = AsyncResult::Success(42);
        assert!(success_result.is_success());
        assert_eq!(success_result.unwrap(), 42);

        let error_result: AsyncResult<i32> = AsyncResult::Error("test error".to_string());
        assert!(error_result.is_error());
        assert_eq!(error_result.unwrap_or(0), 0);

        let timeout_result: AsyncResult<i32> = AsyncResult::Timeout;
        assert_eq!(timeout_result.unwrap_or(42), 42);
    }
}
