use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 应用程序状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppState {
    /// 初始化状态
    Initializing,
    /// 正常运行状态
    Running,
    /// 加载中状态
    Loading,
    /// 设置界面状态
    Settings,
    /// 添加条目状态
    AddingEntry,
    /// 编辑条目状态
    EditingEntry,
    /// 标签管理状态
    TagManager,
    /// 集合管理状态
    CollectionManager,
    /// 导入导出状态
    ImportExport,
    /// 错误状态
    Error(String),
    /// 退出状态
    Exiting,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Initializing
    }
}

/// 状态转换事件
#[derive(Debug, Clone, PartialEq)]
pub enum StateEvent {
    /// 初始化完成
    InitializationComplete,
    /// 进入设置
    EnterSettings,
    /// 退出设置
    ExitSettings,
    /// 开始添加条目
    StartAddingEntry,
    /// 完成添加条目
    FinishAddingEntry,
    /// 取消添加条目
    CancelAddingEntry,
    /// 开始编辑条目
    StartEditingEntry,
    /// 完成编辑条目
    FinishEditingEntry,
    /// 取消编辑条目
    CancelEditingEntry,
    /// 进入标签管理
    EnterTagManager,
    /// 退出标签管理
    ExitTagManager,
    /// 进入集合管理
    EnterCollectionManager,
    /// 退出集合管理
    ExitCollectionManager,
    /// 进入导入导出
    EnterImportExport,
    /// 退出导入导出
    ExitImportExport,
    /// 开始加载
    StartLoading,
    /// 完成加载
    FinishLoading,
    /// 发生错误
    Error(String),
    /// 从错误中恢复
    RecoverFromError,
    /// 退出应用
    Exit,
}

/// 状态转换规则
pub struct StateTransition {
    from: AppState,
    to: AppState,
    event: StateEvent,
}

impl StateTransition {
    pub fn new(from: AppState, to: AppState, event: StateEvent) -> Self {
        Self { from, to, event }
    }
}

/// 状态机
pub struct StateMachine {
    current_state: AppState,
    previous_state: Option<AppState>,
    transitions: Vec<StateTransition>,
    state_history: Vec<AppState>,
    max_history_size: usize,
    state_listeners: Vec<Box<dyn Fn(&AppState, &Option<AppState>) + Send + Sync>>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        let mut state_machine = Self {
            current_state: AppState::Initializing,
            previous_state: None,
            transitions: Vec::new(),
            state_history: Vec::new(),
            max_history_size: 50,
            state_listeners: Vec::new(),
        };

        // 定义状态转换规则
        state_machine.setup_transitions();
        state_machine
    }

    /// 设置状态转换规则
    fn setup_transitions(&mut self) {
        let transitions = vec![
            // 从初始化状态的转换
            StateTransition::new(
                AppState::Initializing,
                AppState::Running,
                StateEvent::InitializationComplete,
            ),
            StateTransition::new(
                AppState::Initializing,
                AppState::Error("初始化失败".to_string()),
                StateEvent::Error("初始化失败".to_string()),
            ),
            // 从运行状态的转换
            StateTransition::new(
                AppState::Running,
                AppState::Settings,
                StateEvent::EnterSettings,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::AddingEntry,
                StateEvent::StartAddingEntry,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::EditingEntry,
                StateEvent::StartEditingEntry,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::TagManager,
                StateEvent::EnterTagManager,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::CollectionManager,
                StateEvent::EnterCollectionManager,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::ImportExport,
                StateEvent::EnterImportExport,
            ),
            StateTransition::new(
                AppState::Running,
                AppState::Loading,
                StateEvent::StartLoading,
            ),
            StateTransition::new(AppState::Running, AppState::Exiting, StateEvent::Exit),
            // 从设置状态的转换
            StateTransition::new(
                AppState::Settings,
                AppState::Running,
                StateEvent::ExitSettings,
            ),
            // 从添加条目状态的转换
            StateTransition::new(
                AppState::AddingEntry,
                AppState::Running,
                StateEvent::FinishAddingEntry,
            ),
            StateTransition::new(
                AppState::AddingEntry,
                AppState::Running,
                StateEvent::CancelAddingEntry,
            ),
            // 从编辑条目状态的转换
            StateTransition::new(
                AppState::EditingEntry,
                AppState::Running,
                StateEvent::FinishEditingEntry,
            ),
            StateTransition::new(
                AppState::EditingEntry,
                AppState::Running,
                StateEvent::CancelEditingEntry,
            ),
            // 从标签管理状态的转换
            StateTransition::new(
                AppState::TagManager,
                AppState::Running,
                StateEvent::ExitTagManager,
            ),
            // 从集合管理状态的转换
            StateTransition::new(
                AppState::CollectionManager,
                AppState::Running,
                StateEvent::ExitCollectionManager,
            ),
            // 从导入导出状态的转换
            StateTransition::new(
                AppState::ImportExport,
                AppState::Running,
                StateEvent::ExitImportExport,
            ),
            // 从加载状态的转换
            StateTransition::new(
                AppState::Loading,
                AppState::Running,
                StateEvent::FinishLoading,
            ),
            // 错误状态的转换
            StateTransition::new(
                AppState::Error("".to_string()),
                AppState::Running,
                StateEvent::RecoverFromError,
            ),
            // 任何状态都可以转换到错误状态
            StateTransition::new(
                AppState::Running,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::Settings,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::AddingEntry,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::EditingEntry,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::TagManager,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::CollectionManager,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::ImportExport,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            StateTransition::new(
                AppState::Loading,
                AppState::Error("".to_string()),
                StateEvent::Error("".to_string()),
            ),
            // 从错误状态到退出状态
            StateTransition::new(
                AppState::Error("".to_string()),
                AppState::Exiting,
                StateEvent::Exit,
            ),
        ];

        self.transitions = transitions;
    }

    /// 获取当前状态
    pub fn current_state(&self) -> &AppState {
        &self.current_state
    }

    /// 获取前一个状态
    pub fn previous_state(&self) -> &Option<AppState> {
        &self.previous_state
    }

    /// 获取状态历史
    pub fn state_history(&self) -> &Vec<AppState> {
        &self.state_history
    }

    /// 处理状态事件
    pub fn handle_event(&mut self, event: StateEvent) -> Result<(), String> {
        let target_state = self.find_target_state(&event)?;
        self.transition_to_state(target_state);
        Ok(())
    }

    /// 查找目标状态
    fn find_target_state(&self, event: &StateEvent) -> Result<AppState, String> {
        // 处理错误事件的特殊情况
        if let StateEvent::Error(msg) = event {
            return Ok(AppState::Error(msg.clone()));
        }

        // 查找匹配的转换规则
        for transition in &self.transitions {
            if self.states_match(&transition.from, &self.current_state)
                && self.events_match(&transition.event, event)
            {
                return Ok(transition.to.clone());
            }
        }

        Err(format!(
            "无效的状态转换: {:?} -> {:?}",
            self.current_state, event
        ))
    }

    /// 检查状态是否匹配
    fn states_match(&self, pattern: &AppState, actual: &AppState) -> bool {
        match (pattern, actual) {
            (AppState::Error(_), AppState::Error(_)) => true,
            _ => std::mem::discriminant(pattern) == std::mem::discriminant(actual),
        }
    }

    /// 检查事件是否匹配
    fn events_match(&self, pattern: &StateEvent, actual: &StateEvent) -> bool {
        match (pattern, actual) {
            (StateEvent::Error(_), StateEvent::Error(_)) => true,
            _ => std::mem::discriminant(pattern) == std::mem::discriminant(actual),
        }
    }

    /// 转换到新状态
    fn transition_to_state(&mut self, new_state: AppState) {
        let old_state = self.current_state.clone();
        self.previous_state = Some(old_state.clone());
        self.current_state = new_state;

        // 添加到历史记录
        self.state_history.push(old_state.clone());
        if self.state_history.len() > self.max_history_size {
            self.state_history.remove(0);
        }

        // 通知监听器
        for listener in &self.state_listeners {
            listener(&self.current_state, &self.previous_state);
        }
    }

    /// 添加状态监听器
    pub fn add_state_listener<F>(&mut self, listener: F)
    where
        F: Fn(&AppState, &Option<AppState>) + Send + Sync + 'static,
    {
        self.state_listeners.push(Box::new(listener));
    }

    /// 检查是否可以处理事件
    pub fn can_handle_event(&self, event: &StateEvent) -> bool {
        self.find_target_state(event).is_ok()
    }

    /// 强制设置状态（谨慎使用）
    pub fn force_state(&mut self, state: AppState) {
        self.transition_to_state(state);
    }

    /// 回到上一个状态
    pub fn go_back(&mut self) -> Result<(), String> {
        if let Some(prev_state) = self.previous_state.clone() {
            self.transition_to_state(prev_state);
            Ok(())
        } else {
            Err("没有可以返回的状态".to_string())
        }
    }

    /// 检查是否处于特定状态
    pub fn is_in_state(&self, state: &AppState) -> bool {
        std::mem::discriminant(&self.current_state) == std::mem::discriminant(state)
    }

    /// 检查是否处于错误状态
    pub fn is_in_error_state(&self) -> bool {
        matches!(self.current_state, AppState::Error(_))
    }

    /// 获取错误信息（如果处于错误状态）
    pub fn get_error_message(&self) -> Option<&str> {
        match &self.current_state {
            AppState::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

/// 状态管理器 - 线程安全的状态机包装器
#[derive(Clone)]
pub struct StateManager {
    state_machine: Arc<Mutex<StateMachine>>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state_machine: Arc::new(Mutex::new(StateMachine::new())),
        }
    }

    /// 获取当前状态
    pub fn current_state(&self) -> AppState {
        self.state_machine.lock().unwrap().current_state().clone()
    }

    /// 处理状态事件
    pub fn handle_event(&self, event: StateEvent) -> Result<(), String> {
        self.state_machine.lock().unwrap().handle_event(event)
    }

    /// 检查是否可以处理事件
    pub fn can_handle_event(&self, event: &StateEvent) -> bool {
        self.state_machine.lock().unwrap().can_handle_event(event)
    }

    /// 强制设置状态
    pub fn force_state(&self, state: AppState) {
        self.state_machine.lock().unwrap().force_state(state);
    }

    /// 回到上一个状态
    pub fn go_back(&self) -> Result<(), String> {
        self.state_machine.lock().unwrap().go_back()
    }

    /// 检查是否处于特定状态
    pub fn is_in_state(&self, state: &AppState) -> bool {
        self.state_machine.lock().unwrap().is_in_state(state)
    }

    /// 检查是否处于错误状态
    pub fn is_in_error_state(&self) -> bool {
        self.state_machine.lock().unwrap().is_in_error_state()
    }

    /// 获取错误信息
    pub fn get_error_message(&self) -> Option<String> {
        self.state_machine
            .lock()
            .unwrap()
            .get_error_message()
            .map(|s| s.to_string())
    }

    /// 添加状态监听器
    pub fn add_state_listener<F>(&self, listener: F)
    where
        F: Fn(&AppState, &Option<AppState>) + Send + Sync + 'static,
    {
        self.state_machine
            .lock()
            .unwrap()
            .add_state_listener(listener);
    }

    /// 获取状态历史
    pub fn get_state_history(&self) -> Vec<AppState> {
        self.state_machine.lock().unwrap().state_history().clone()
    }
}

/// 状态上下文 - 为不同状态提供上下文信息
#[derive(Debug, Clone)]
pub struct StateContext {
    pub data: HashMap<String, String>,
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StateContext {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_initialization() {
        let state_machine = StateMachine::new();
        assert_eq!(*state_machine.current_state(), AppState::Initializing);
    }

    #[test]
    fn test_valid_state_transition() {
        let mut state_machine = StateMachine::new();

        // 从初始化状态转换到运行状态
        let result = state_machine.handle_event(StateEvent::InitializationComplete);
        assert!(result.is_ok());
        assert_eq!(*state_machine.current_state(), AppState::Running);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut state_machine = StateMachine::new();

        // 尝试从初始化状态直接转换到设置状态（无效）
        let result = state_machine.handle_event(StateEvent::EnterSettings);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_state_transition() {
        let mut state_machine = StateMachine::new();

        // 转换到运行状态
        state_machine
            .handle_event(StateEvent::InitializationComplete)
            .unwrap();

        // 转换到错误状态
        let error_msg = "测试错误".to_string();
        let result = state_machine.handle_event(StateEvent::Error(error_msg.clone()));
        assert!(result.is_ok());
        assert!(state_machine.is_in_error_state());
        assert_eq!(state_machine.get_error_message(), Some(error_msg.as_str()));
    }

    #[test]
    fn test_state_history() {
        let mut state_machine = StateMachine::new();

        // 执行几个状态转换
        state_machine
            .handle_event(StateEvent::InitializationComplete)
            .unwrap();
        state_machine
            .handle_event(StateEvent::EnterSettings)
            .unwrap();
        state_machine
            .handle_event(StateEvent::ExitSettings)
            .unwrap();

        let history = state_machine.state_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], AppState::Initializing);
        assert_eq!(history[1], AppState::Running);
        assert_eq!(history[2], AppState::Settings);
    }

    #[test]
    fn test_state_manager_thread_safety() {
        let state_manager = StateManager::new();

        // 测试多线程访问
        let manager_clone = state_manager.clone();
        let handle = std::thread::spawn(move || {
            manager_clone.handle_event(StateEvent::InitializationComplete)
        });

        let result = handle.join().unwrap();
        assert!(result.is_ok());
        assert_eq!(state_manager.current_state(), AppState::Running);
    }

    #[test]
    fn test_state_context() {
        let mut context = StateContext::new();

        context.set("key1", "value1");
        context.set("key2", "value2");

        assert_eq!(context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.get("key2"), Some(&"value2".to_string()));
        assert_eq!(context.get("key3"), None);

        context.remove("key1");
        assert_eq!(context.get("key1"), None);

        context.clear();
        assert_eq!(context.get("key2"), None);
    }
}
