use std::sync::Arc;

use crate::core::{Result, EidosError, SourceLocation};
use crate::core::ast::{ASTNode, Node, Program};
use super::registry::DSLRegistry;
use super::extension::DSLExtension;

/// DSLブロックの処理を行うプロセッサ
pub struct DSLProcessor;

impl DSLProcessor {
    pub fn new() -> Self {
        Self
    }
    
    /// DSLブロックを処理
    pub fn process_dsl_block(&self, name: &str, content: &str, program: &Program, location: SourceLocation) -> Result<ASTNode> {
        // レジストリからDSL拡張を取得
        let registry = DSLRegistry::global().read().unwrap();
        let extension = registry.get(name).ok_or_else(|| {
            EidosError::DSL {
                message: format!("DSL拡張 '{}' が見つかりません", name),
                dsl_name: name.to_string(),
            }
        })?;
        
        // DSL拡張を使ってブロックを処理
        let ast_node = extension.process_block(content, program)?;
        
        // ノードに位置情報を設定
        let node_with_location = ASTNode {
            id: ast_node.id,
            kind: ast_node.kind,
            location,
            type_info: ast_node.type_info,
        };
        
        Ok(node_with_location)
    }
    
    /// 特定のDSL拡張が利用可能かどうか
    pub fn is_dsl_available(&self, name: &str) -> bool {
        let registry = DSLRegistry::global().read().unwrap();
        registry.has_extension(name)
    }
    
    /// 利用可能なすべてのDSL拡張の名前を取得
    pub fn list_available_dsls(&self) -> Vec<String> {
        let registry = DSLRegistry::global().read().unwrap();
        registry.list_extensions()
    }
    
    /// 新しいDSL拡張を登録
    pub fn register_dsl(&self, name: String, extension: Arc<dyn DSLExtension>) {
        let mut registry = DSLRegistry::global().write().unwrap();
        registry.register(name, extension);
    }
} 