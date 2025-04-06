use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::SourceLocation;
use super::types::Type;
use super::symbol::SymbolId;

/// ASTノードの識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// AST要素が持つ型情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    /// 推論前または推論不能
    Unknown,
    /// 推論済み
    Resolved(Type),
    /// 明示的に指定された型
    Explicit(Type),
}

/// AST中の単項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // !
    BitNot, // ~
}

/// AST中の二項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // 算術演算子
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %
    
    // ビット演算子
    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    LShift,   // <<
    RShift,   // >>
    
    // 比較演算子
    Eq,       // ==
    NotEq,    // !=
    Lt,       // <
    LtEq,     // <=
    Gt,       // >
    GtEq,     // >=
    
    // 論理演算子
    And,      // &&
    Or,       // ||
}

/// リテラル値
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Unit,
}

/// ASTノード
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // リテラル
    Literal(Literal),
    
    // 識別子参照
    Identifier {
        name: String,
        symbol: Option<SymbolId>,
    },
    
    // 単項演算式
    UnaryExpr {
        op: UnaryOp,
        expr: Box<ASTNode>,
    },
    
    // 二項演算式
    BinaryExpr {
        op: BinaryOp,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    
    // 条件式（if-then-else）
    IfExpr {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    
    // ブロック式
    BlockExpr {
        statements: Vec<ASTNode>,
        result: Option<Box<ASTNode>>,
    },
    
    // 変数宣言
    VarDecl {
        name: String,
        symbol: Option<SymbolId>,
        type_annotation: Option<Type>,
        initializer: Option<Box<ASTNode>>,
        is_mutable: bool,
    },
    
    // 関数定義
    FunctionDef {
        name: String,
        symbol: Option<SymbolId>,
        params: Vec<FunctionParam>,
        return_type: Option<Type>,
        body: Box<ASTNode>,
    },
    
    // 関数呼び出し
    FunctionCall {
        callee: Box<ASTNode>,
        args: Vec<ASTNode>,
    },
    
    // 代入
    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
    },
    
    // ループ
    WhileLoop {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    
    // 型定義
    TypeDef {
        name: String,
        symbol: Option<SymbolId>,
        definition: Type,
    },
    
    // DSLブロック
    DSLBlock {
        name: String,
        content: String,
        processed_ast: Option<Box<ASTNode>>,
    },
}

/// 関数パラメータ
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub symbol: Option<SymbolId>,
    pub param_type: Option<Type>,
}

/// 完全なASTノード（メタデータ付き）
#[derive(Debug, Clone, PartialEq)]
pub struct ASTNode {
    pub id: NodeId,
    pub kind: Node,
    pub location: SourceLocation,
    pub type_info: TypeInfo,
}

impl ASTNode {
    pub fn new(kind: Node, location: SourceLocation) -> Self {
        static mut NEXT_ID: usize = 0;
        
        let id = unsafe {
            let id = NodeId(NEXT_ID);
            NEXT_ID += 1;
            id
        };
        
        Self {
            id,
            kind,
            location,
            type_info: TypeInfo::Unknown,
        }
    }
    
    pub fn with_type(mut self, type_info: TypeInfo) -> Self {
        self.type_info = type_info;
        self
    }
}

/// プログラム全体のAST
#[derive(Debug, Clone)]
pub struct Program {
    pub nodes: Vec<ASTNode>,
    pub node_map: HashMap<NodeId, ASTNode>,
    pub file_path: String,
}

impl Program {
    pub fn new(file_path: String) -> Self {
        Self {
            nodes: Vec::new(),
            node_map: HashMap::new(),
            file_path,
        }
    }
    
    pub fn add_node(&mut self, node: ASTNode) -> NodeId {
        let id = node.id;
        self.nodes.push(node.clone());
        self.node_map.insert(id, node);
        id
    }
    
    pub fn get_node(&self, id: NodeId) -> Option<&ASTNode> {
        self.node_map.get(&id)
    }
} 