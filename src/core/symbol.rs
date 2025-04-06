use std::collections::HashMap;
use std::fmt;

/// シンボル識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.0)
    }
}

/// シンボルのスコープ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,     // グローバルスコープ
    Function,   // 関数スコープ
    Block,      // ブロックスコープ
    Module,     // モジュールスコープ
}

/// 識別子の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,  // 変数
    Function,  // 関数
    Parameter, // 関数パラメータ
    Type,      // 型定義
    Module,    // モジュール
    EnumVariant, // 列挙型バリアント
    DSLSymbol, // DSL拡張シンボル
}

/// シンボル情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub scope_id: ScopeId,
    pub is_mutable: bool,
    pub is_exported: bool,
}

/// スコープ識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope({})", self.0)
    }
}

/// スコープ情報
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
}

/// シンボルテーブル
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<SymbolId, Symbol>,
    scopes: HashMap<ScopeId, Scope>,
    current_scope: ScopeId,
    next_symbol_id: usize,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            symbols: HashMap::new(),
            scopes: HashMap::new(),
            current_scope: ScopeId(0),
            next_symbol_id: 0,
            next_scope_id: 0,
        };
        
        // グローバルスコープを作成
        let global_scope_id = table.create_scope(ScopeKind::Global, None);
        table.current_scope = global_scope_id;
        
        table
    }
    
    /// 新しいスコープを作成
    pub fn create_scope(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        
        let scope = Scope {
            id,
            kind,
            parent,
            symbols: HashMap::new(),
        };
        
        self.scopes.insert(id, scope);
        id
    }
    
    /// 現在のスコープを取得
    pub fn current_scope(&self) -> &Scope {
        self.scopes.get(&self.current_scope).unwrap()
    }
    
    /// スコープを入力（新しいスコープを作成して現在のスコープに設定）
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let parent = self.current_scope;
        let new_scope_id = self.create_scope(kind, Some(parent));
        self.current_scope = new_scope_id;
        new_scope_id
    }
    
    /// スコープを抜ける（親スコープに戻る）
    pub fn exit_scope(&mut self) -> Result<(), String> {
        let current = self.current_scope();
        match current.parent {
            Some(parent_id) => {
                self.current_scope = parent_id;
                Ok(())
            },
            None => Err("グローバルスコープから抜けることはできません".to_string()),
        }
    }
    
    /// 新しいシンボルを宣言
    pub fn declare_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        is_mutable: bool,
        is_exported: bool,
    ) -> Result<SymbolId, String> {
        // 現在のスコープで同名のシンボルがないかチェック
        let current_scope = self.current_scope;
        let scope = self.scopes.get(&current_scope).unwrap();
        
        if scope.symbols.contains_key(&name) {
            return Err(format!("シンボル '{}' は現在のスコープですでに宣言されています", name));
        }
        
        // 新しいシンボルを作成
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        
        let symbol = Symbol {
            id,
            name: name.clone(),
            kind,
            scope_id: current_scope,
            is_mutable,
            is_exported,
        };
        
        // シンボルテーブルに追加
        self.symbols.insert(id, symbol);
        
        // スコープにシンボルを登録
        let scope = self.scopes.get_mut(&current_scope).unwrap();
        scope.symbols.insert(name, id);
        
        Ok(id)
    }
    
    /// シンボルを検索
    pub fn lookup_symbol(&self, name: &str) -> Option<SymbolId> {
        let mut current_scope_id = Some(self.current_scope);
        
        // 現在のスコープから親スコープへと順に探す
        while let Some(scope_id) = current_scope_id {
            let scope = self.scopes.get(&scope_id).unwrap();
            
            if let Some(symbol_id) = scope.symbols.get(name) {
                return Some(*symbol_id);
            }
            
            current_scope_id = scope.parent;
        }
        
        None
    }
    
    /// シンボルIDからシンボル情報を取得
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }
    
    /// スコープIDからスコープ情報を取得
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }
} 