use std::any::Any;

use crate::core::Result;
use crate::core::ast::{ASTNode, Program};
use crate::core::types::Type;

/// DSL拡張のトレイト
pub trait DSLExtension: Send + Sync {
    /// DSL拡張の名前を取得
    fn name(&self) -> &str;
    
    /// DSL拡張の説明を取得
    fn description(&self) -> &str;
    
    /// DSLブロックをASTに変換
    fn process_block(&self, content: &str, program: &Program) -> Result<ASTNode>;
    
    /// DSL固有の型を登録
    fn register_types(&self) -> Vec<(String, Type)>;
    
    /// DSL固有のシンボルを登録
    fn register_builtins(&self) -> Vec<String>;
    
    /// このDSLがサポートするカスタムディレクティブを取得
    fn supported_directives(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// カスタムデータにアクセス（実装固有の拡張用）
    fn as_any(&self) -> &dyn Any;
} 