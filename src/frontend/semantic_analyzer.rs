use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::{Result, EidosError, SourceLocation};
use crate::core::ast::{ASTNode, Node, Program, NodeId};
use crate::core::symbol::{SymbolTable, SymbolId, SymbolKind, ScopeKind};

/// 意味解析器
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    node_symbols: HashMap<NodeId, SymbolId>,
}

impl SemanticAnalyzer {
    /// 新しい意味解析器を作成
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            node_symbols: HashMap::new(),
        }
    }
    
    /// ASTの意味解析を実行
    pub fn analyze(&mut self, program: Program) -> Result<Program> {
        // 意味解析の開始をログに記録
        info!("意味解析を実行中: {} ノード", program.node_count());
        
        // グローバルスコープに入る
        self.enter_scope(ScopeKind::Global);
        
        // 1. 最初にすべての関数と構造体の宣言をシンボルテーブルに登録（相互再帰対応）
        self.register_declarations(&program)?;
        debug!("宣言の登録完了: {} シンボル", self.symbol_table.symbol_count());
        
        // 2. モジュールのインポート処理
        self.process_imports(&program)?;
        
        // 3. 各ノードの意味解析を行う（トップダウンで処理）
        for node_id in program.traverse_pre_order() {
            let node = program.get_node(node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: ID={:?}", node_id))
            })?;
            
            trace!("ノード分析中: {:?} at {}:{}", node.kind, node.location.line, node.location.column);
            self.analyze_node(&program, node_id, node)?;
        }
        
        // 4. 未使用の変数や関数に関する警告を生成
        self.check_unused_symbols()?;
        
        // 5. 到達不能コードの検出
        self.detect_unreachable_code(&program)?;
        
        // グローバルスコープから出る
        self.exit_scope()?;
        
        // シンボル解決情報を含むプログラムを生成
        let mut analyzed_program = program.clone();
        
        // 各ノードにシンボル情報を付与
        for (node_id, symbol_id) in &self.node_symbols {
            if let Some(node) = analyzed_program.get_node_mut(*node_id) {
                node.symbol_id = Some(*symbol_id);
            }
        }
        
        // 型情報の検証と最適化のヒントを追加
        self.annotate_optimization_hints(&mut analyzed_program)?;
        
        info!("意味解析完了: 検証済みノード数={}", self.node_symbols.len());
        Ok(analyzed_program)
    }
    
    /// モジュールのインポート処理
    fn process_imports(&mut self, program: &Program) -> Result<()> {
        // インポート文を探して処理
        for node_id in program.traverse_pre_order() {
            if let Some(node) = program.get_node(node_id) {
                if let Node::Import { path, symbols } = &node.kind {
                    debug!("インポート処理: {}", path);
                    
                    // モジュールパスの解決
                    let module_path = self.resolve_module_path(path)?;
                    
                    // モジュールの読み込み
                    let module = self.load_module(&module_path, &node.location)?;
                    
                    // ワイルドカードインポートかどうかを確認
                    let is_wildcard_import = symbols.iter().any(|s| s == "*");
                    
                    if is_wildcard_import {
                        // モジュール内のすべての公開シンボルをインポート
                        for (name, symbol_info) in module.get_public_symbols() {
                            // 名前の衝突チェック
                            if self.symbol_table.lookup_in_current_scope(&name).is_some() {
                                return Err(EidosError::Semantic {
                                    message: format!("シンボル '{}' は現在のスコープで既に定義されています", name),
                                    file: node.location.file.clone(),
                                    line: node.location.line,
                                    column: node.location.column,
                                });
                            }
                            
                            // シンボルを現在のスコープに追加
                            let symbol_id = self.declare_symbol(
                                name.clone(),
                                symbol_info.kind.clone(),
                                symbol_info.is_mutable,
                                false // インポートされたシンボルは再エクスポートできない
                            )?;
                            
                            // シンボルに型情報を設定
                            if let Some(type_id) = symbol_info.type_id {
                                self.symbol_table.set_symbol_type(symbol_id, type_id)?;
                            }
                            
                            // ノードとシンボルの関連付け
                            self.node_symbols.insert(node_id, symbol_id);
                        }
                    } else {
                        // 指定されたシンボルのみをインポート
                        for symbol_name in symbols {
                            // モジュールからシンボル情報を取得
                            let symbol_info = module.get_symbol(symbol_name).ok_or_else(|| {
                                EidosError::Semantic {
                                    message: format!("シンボル '{}' はモジュール '{}' に存在しません", symbol_name, path),
                                    file: node.location.file.clone(),
                                    line: node.location.line,
                                    column: node.location.column,
                                }
                            })?;
                            
                            // シンボルが公開されているか確認
                            if !symbol_info.is_public {
                                return Err(EidosError::Semantic {
                                    message: format!("シンボル '{}' はモジュール '{}' で公開されていません", symbol_name, path),
                                    file: node.location.file.clone(),
                                    line: node.location.line,
                                    column: node.location.column,
                                });
                            }
                            
                            // 名前の衝突チェック
                            if self.symbol_table.lookup_in_current_scope(symbol_name).is_some() {
                                return Err(EidosError::Semantic {
                                    message: format!("シンボル '{}' は現在のスコープで既に定義されています", symbol_name),
                                    file: node.location.file.clone(),
                                    line: node.location.line,
                                    column: node.location.column,
                                });
                            }
                            
                            // シンボルを現在のスコープに追加
                            let symbol_id = self.declare_symbol(
                                symbol_name.clone(),
                                symbol_info.kind.clone(),
                                symbol_info.is_mutable,
                                false // インポートされたシンボルは再エクスポートできない
                            )?;
                            
                            // シンボルに型情報を設定
                            if let Some(type_id) = symbol_info.type_id {
                                self.symbol_table.set_symbol_type(symbol_id, type_id)?;
                            }
                            
                            // ノードとシンボルの関連付け
                            self.node_symbols.insert(node_id, symbol_id);
                            
                            // 使用状況の追跡
                            self.symbol_table.mark_as_used(symbol_id);
                        }
                    }
                    
                    // インポートモジュールの依存関係を記録
                    self.record_module_dependency(program.module_name.clone(), module_path);
                }
            }
        }
        Ok(())
    }
    /// 未使用のシンボルをチェック
    fn check_unused_symbols(&self) -> Result<()> {
        let unused_symbols = self.symbol_table.get_unused_symbols();
        
        for (symbol_id, name) in unused_symbols {
            // 警告を生成（エラーではない）
            warn!("未使用のシンボル: {} (ID: {:?})", name, symbol_id);
        }
        
        Ok(())
    }
    
    /// 到達不能コードの検出
    fn detect_unreachable_code(&self, program: &Program) -> Result<()> {
        // 関数内の各ブロックについて、return文の後のコードを検出
        for node_id in program.traverse_pre_order() {
            if let Some(node) = program.get_node(node_id) {
                if let Node::Block { statements } = &node.kind {
                    let mut found_return = false;
                    
                    for (i, stmt_id) in statements.iter().enumerate() {
                        if let Some(stmt) = program.get_node(*stmt_id) {
                            if found_return {
                                warn!("到達不能コード検出: {}:{}", 
                                      stmt.location.line, 
                                      stmt.location.column);
                            }
                            
                            if let Node::Return { .. } = stmt.kind {
                                found_return = true;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 最適化のためのヒントをプログラムに注釈として追加
    fn annotate_optimization_hints(&self, program: &mut Program) -> Result<()> {
        // 定数畳み込みの機会を検出
        for node_id in program.traverse_pre_order() {
            if let Some(node) = program.get_node_mut(node_id) {
                if let Node::BinaryOp { op, left, right } = &node.kind {
                    // 両方のオペランドが定数の場合、最適化のヒントを追加
                    if self.is_constant_expression(program, *left) && 
                       self.is_constant_expression(program, *right) {
                        node.optimization_hints.push("constant_folding".to_string());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 式が定数かどうかを判定
    fn is_constant_expression(&self, program: &Program, node_id: NodeId) -> bool {
        if let Some(node) = program.get_node(node_id) {
            match &node.kind {
                Node::IntLiteral(_) | Node::FloatLiteral(_) | 
                Node::BoolLiteral(_) | Node::StringLiteral(_) => true,
                _ => false
            }
        } else {
            false
        }
    }
}
    
    /// AST内の関数と構造体の宣言を事前登録
    fn register_declarations(&mut self, program: &Program) -> Result<()> {
        for node_id in program.traverse_pre_order() {
            let node = program.get_node(node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            match &node.kind {
                Node::Function { name, parameters: _, return_type: _, body: _ } => {
                    // 関数の宣言をシンボルテーブルに登録
                    let symbol_id = self.declare_symbol(
                        name.clone(),
                        SymbolKind::Function,
                        false, // 関数は変更不可
                        false, // デフォルトではエクスポートしない
                    )?;
                    
                    self.node_symbols.insert(node_id, symbol_id);
                },
                Node::Struct { name, fields: _, methods: _ } => {
                    // 構造体の宣言をシンボルテーブルに登録
                    let symbol_id = self.declare_symbol(
                        name.clone(),
                        SymbolKind::Type,
                        false, // 型は変更不可
                        false, // デフォルトではエクスポートしない
                    )?;
                    
                    self.node_symbols.insert(node_id, symbol_id);
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// ノードの意味解析
    fn analyze_node(&mut self, program: &Program, node_id: NodeId, node: &ASTNode) -> Result<()> {
        match &node.kind {
            Node::Variable(name) => {
                // 変数参照の解決
                if let Some(symbol_id) = self.resolve_identifier(name) {
                    self.node_symbols.insert(node_id, symbol_id);
                } else {
                    return Err(EidosError::Semantic {
                        message: format!("未定義の識別子: {}", name),
                        file: node.location.file.clone(),
                        line: node.location.line,
                        column: node.location.column,
                    });
                }
            },
            Node::Let { name, type_annotation: _, initializer } => {
                // 変数宣言をシンボルテーブルに登録
                let symbol_id = self.declare_symbol(
                    name.clone(),
                    SymbolKind::Variable,
                    true, // デフォルトでは可変
                    false, // デフォルトではエクスポートしない
                )?;
                
                self.node_symbols.insert(node_id, symbol_id);
                
                // 初期化子があれば解析
                if let Some(init_id) = initializer {
                    let init_node = program.get_node(*init_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", init_id))
                    })?;
                    
                    self.analyze_node(program, *init_id, init_node)?;
                }
            },
            Node::Assign { target, value } => {
                // 代入式の解析
                
                // まず対象を解析
                let target_node = program.get_node(*target).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", target))
                })?;
                
                self.analyze_node(program, *target, target_node)?;
                
                // 代入対象が変数の場合、可変かチェック
                if let Node::Variable(name) = &target_node.kind {
                    if let Some(symbol_id) = self.resolve_identifier(name) {
                        let symbol = self.symbol_table.get_symbol(symbol_id).ok_or_else(|| {
                            EidosError::Internal(format!("シンボルが見つかりません: {:?}", symbol_id))
                        })?;
                        
                        if !symbol.is_mutable {
                            return Err(EidosError::Semantic {
                                message: format!("イミュータブルな変数に代入しようとしています: {}", name),
                                file: node.location.file.clone(),
                                line: node.location.line,
                                column: node.location.column,
                            });
                        }
                    }
                }
                
                // 値を解析
                let value_node = program.get_node(*value).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", value))
                })?;
                
                self.analyze_node(program, *value, value_node)?;
            },
            Node::Function { name, parameters, return_type: _, body } => {
                // 関数スコープに入る
                self.enter_scope(ScopeKind::Function);
                
                // パラメータをシンボルテーブルに登録
                for param in parameters {
                    let symbol_id = self.declare_symbol(
                        param.name.clone(),
                        SymbolKind::Parameter,
                        false, // パラメータはイミュータブル
                        false, // エクスポートしない
                    )?;
                    
                    if let Some(param_node_id) = param.node_id {
                        self.node_symbols.insert(param_node_id, symbol_id);
                    }
                }
                
                // 関数本体を解析
                let body_node = program.get_node(*body).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", body))
                })?;
                
                self.analyze_node(program, *body, body_node)?;
                
                // 関数スコープから出る
                self.exit_scope()?;
            },
            Node::Block { statements, expression } => {
                // ブロックスコープに入る
                self.enter_scope(ScopeKind::Block);
                
                // 各文を解析
                for stmt_id in statements {
                    let stmt_node = program.get_node(*stmt_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", stmt_id))
                    })?;
                    
                    self.analyze_node(program, *stmt_id, stmt_node)?;
                }
                
                // 最後の式があれば解析
                if let Some(expr_id) = expression {
                    let expr_node = program.get_node(*expr_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", expr_id))
                    })?;
                    
                    self.analyze_node(program, *expr_id, expr_node)?;
                }
                
                // ブロックスコープから出る
                self.exit_scope()?;
            },
            Node::If { condition, then_branch, else_branch } => {
                // 条件式を解析
                let cond_node = program.get_node(*condition).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", condition))
                })?;
                
                self.analyze_node(program, *condition, cond_node)?;
                
                // then節を解析
                let then_node = program.get_node(*then_branch).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", then_branch))
                })?;
                
                self.analyze_node(program, *then_branch, then_node)?;
                
                // else節があれば解析
                if let Some(else_id) = else_branch {
                    let else_node = program.get_node(*else_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", else_id))
                    })?;
                    
                    self.analyze_node(program, *else_id, else_node)?;
                }
            },
            Node::Call { callee, arguments } => {
                // 呼び出し対象を解析
                let callee_node = program.get_node(*callee).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", callee))
                })?;
                
                self.analyze_node(program, *callee, callee_node)?;
                
                // 引数を解析
                for arg_id in arguments {
                    let arg_node = program.get_node(*arg_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", arg_id))
                    })?;
                    
                    self.analyze_node(program, *arg_id, arg_node)?;
                }
            },
            Node::BinaryOp { op: _, left, right } => {
                // 左辺を解析
                let left_node = program.get_node(*left).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", left))
                })?;
                
                self.analyze_node(program, *left, left_node)?;
                
                // 右辺を解析
                let right_node = program.get_node(*right).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", right))
                })?;
                
                self.analyze_node(program, *right, right_node)?;
            },
            Node::UnaryOp { op: _, operand } => {
                // オペランドを解析
                let operand_node = program.get_node(*operand).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", operand))
                })?;
                
                self.analyze_node(program, *operand, operand_node)?;
            },
            Node::Return { value } => {
                // 現在の関数スコープをチェック
                if !self.symbol_table.is_in_function_scope() {
                    return Err(EidosError::Semantic {
                        message: "関数外でreturn文は使用できません".to_string(),
                        file: node.location.file.clone(),
                        line: node.location.line,
                        column: node.location.column,
                    });
                }
                
                // 戻り値があれば解析
                if let Some(value_id) = value {
                    let value_node = program.get_node(*value_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", value_id))
                    })?;
                    
                    self.analyze_node(program, *value_id, value_node)?;
                }
            },
            Node::Struct { name: _, fields, methods } => {
                // フィールドをシンボルテーブルに登録
                for field in fields {
                    let symbol_id = self.declare_symbol(
                        field.name.clone(),
                        SymbolKind::Field,
                        field.is_mutable,
                        field.is_public,
                    )?;
                    
                    if let Some(field_node_id) = field.node_id {
                        self.node_symbols.insert(field_node_id, symbol_id);
                    }
                }
                
                // メソッドを解析
                for method_id in methods {
                    let method_node = program.get_node(*method_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", method_id))
                    })?;
                    
                    self.analyze_node(program, *method_id, method_node)?;
                }
            },
            Node::Literal(_) => {
                // リテラルは特に意味解析は不要
            },
            _ => {
                // その他のノード型の意味解析（必要に応じて実装）
            }
        }
        
        Ok(())
    }
    
    /// シンボルを宣言
    fn declare_symbol(&mut self, name: String, kind: SymbolKind, is_mutable: bool, is_exported: bool) -> Result<SymbolId> {
        self.symbol_table.declare_symbol(name, kind, is_mutable, is_exported)
            .map_err(|e| {
                EidosError::Semantic {
                    message: e,
                    file: PathBuf::from("<unknown>"),
                    line: 0,
                    column: 0,
                }
            })
    }
    
    /// 識別子を解決
    fn resolve_identifier(&self, name: &str) -> Option<SymbolId> {
        self.symbol_table.lookup_symbol(name)
    }
    
    /// スコープを作成
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.symbol_table.enter_scope(kind);
    }
    
    /// スコープを抜ける
    fn exit_scope(&mut self) -> Result<()> {
        self.symbol_table.exit_scope()
            .map_err(|e| {
                EidosError::Semantic {
                    message: e,
                    file: PathBuf::from("<unknown>"),
                    line: 0,
                    column: 0,
                }
            })
    }
}

// 必要なインポートとログマクロの定義
use std::path::PathBuf;
use log::{info as log_info, warn, error, debug, trace};

/// ログ出力用マクロ - 情報レベルのログを出力
macro_rules! info {
    ($($arg:tt)*) => {
        log_info!("{}", format!($($arg)*));
    };
}

/// ログ出力用マクロ - 警告レベルのログを出力
macro_rules! warn {
    ($($arg:tt)*) => {
        warn!("{}", format!($($arg)*));
    };
}

/// ログ出力用マクロ - エラーレベルのログを出力
macro_rules! error {
    ($($arg:tt)*) => {
        error!("{}", format!($($arg)*));
    };
}

/// ログ出力用マクロ - デバッグレベルのログを出力
macro_rules! debug {
    ($($arg:tt)*) => {
        debug!("{}", format!($($arg)*));
    };
}

/// ログ出力用マクロ - トレースレベルのログを出力
macro_rules! trace {
    ($($arg:tt)*) => {
        trace!("{}", format!($($arg)*));
    };
}