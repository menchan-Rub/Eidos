use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::Result;
use super::extension::DSLExtension;

/// DSL拡張を管理するレジストリ
pub struct DSLRegistry {
    extensions: HashMap<String, Arc<dyn DSLExtension>>,
}

// シングルトンパターンでDSLレジストリを実装
lazy_static::lazy_static! {
    static ref REGISTRY: RwLock<DSLRegistry> = RwLock::new(DSLRegistry::new());
}

impl DSLRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }
    
    /// グローバルレジストリを取得
    pub fn global() -> &'static RwLock<DSLRegistry> {
        &REGISTRY
    }
    
    /// DSL拡張を登録
    pub fn register(&mut self, name: String, extension: Arc<dyn DSLExtension>) {
        self.extensions.insert(name, extension);
    }
    
    /// DSL拡張を取得
    pub fn get(&self, name: &str) -> Option<Arc<dyn DSLExtension>> {
        self.extensions.get(name).cloned()
    }
    
    /// すべてのDSL拡張の名前を取得
    pub fn list_extensions(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
    
    /// DSL拡張が存在するかどうか
    pub fn has_extension(&self, name: &str) -> bool {
        self.extensions.contains_key(name)
    }
    
    /// DSL拡張を削除
    pub fn unregister(&mut self, name: &str) -> bool {
        self.extensions.remove(name).is_some()
    }
} 