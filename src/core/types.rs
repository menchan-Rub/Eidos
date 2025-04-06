use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::symbol::SymbolId;

/// 型識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type({})", self.0)
    }
}

/// 型パラメータ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParam {
    pub name: String,
    pub constraints: Vec<Type>,
}

/// 型のバリエーション
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    // 基本型
    Unit,
    Bool,
    Int,
    Float,
    Char,
    String,
    
    // 複合型
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    // ユーザー定義型
    Struct {
        name: String,
        fields: Vec<StructField>,
        type_params: Vec<TypeParam>,
    },
    
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        type_params: Vec<TypeParam>,
    },
    
    // 型参照
    TypeRef {
        name: String,
        symbol: Option<SymbolId>,
    },
    
    // 型パラメータ参照
    TypeParam {
        name: String,
    },
    
    // DSLカスタム型
    DSLType {
        name: String,
        dsl_name: String,
        custom_data: Option<Rc<dyn std::any::Any>>,
    },
    
    // 未知の型（型推論中に使用）
    Unknown,
    
    // エラー型（型エラー時に使用）
    Error,
}

/// 型を表す構造体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub id: TypeId,
    pub kind: TypeKind,
}

impl Type {
    pub fn new(kind: TypeKind) -> Self {
        static mut NEXT_ID: usize = 0;
        
        let id = unsafe {
            let id = TypeId(NEXT_ID);
            NEXT_ID += 1;
            id
        };
        
        Self { id, kind }
    }
    
    /// 基本的な組み込み型を作成するヘルパーメソッド
    pub fn unit() -> Self {
        Self::new(TypeKind::Unit)
    }
    
    pub fn bool() -> Self {
        Self::new(TypeKind::Bool)
    }
    
    pub fn int() -> Self {
        Self::new(TypeKind::Int)
    }
    
    pub fn float() -> Self {
        Self::new(TypeKind::Float)
    }
    
    pub fn char() -> Self {
        Self::new(TypeKind::Char)
    }
    
    pub fn string() -> Self {
        Self::new(TypeKind::String)
    }
    
    pub fn array(element_type: Type) -> Self {
        Self::new(TypeKind::Array(Box::new(element_type)))
    }
    
    pub fn tuple(element_types: Vec<Type>) -> Self {
        Self::new(TypeKind::Tuple(element_types))
    }
    
    pub fn function(params: Vec<Type>, return_type: Type) -> Self {
        Self::new(TypeKind::Function {
            params,
            return_type: Box::new(return_type),
        })
    }
    
    pub fn type_ref(name: String) -> Self {
        Self::new(TypeKind::TypeRef {
            name,
            symbol: None,
        })
    }
    
    pub fn unknown() -> Self {
        Self::new(TypeKind::Unknown)
    }
    
    pub fn error() -> Self {
        Self::new(TypeKind::Error)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeKind::Unit => write!(f, "()"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Int => write!(f, "int"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::String => write!(f, "string"),
            TypeKind::Array(elem) => write!(f, "[{}]", elem),
            TypeKind::Tuple(elems) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            },
            TypeKind::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            TypeKind::Struct { name, .. } => write!(f, "{}", name),
            TypeKind::Enum { name, .. } => write!(f, "{}", name),
            TypeKind::TypeRef { name, .. } => write!(f, "{}", name),
            TypeKind::TypeParam { name } => write!(f, "{}", name),
            TypeKind::DSLType { name, dsl_name, .. } => write!(f, "{}:{}", dsl_name, name),
            TypeKind::Unknown => write!(f, "?"),
            TypeKind::Error => write!(f, "<error>"),
        }
    }
}

/// 構造体のフィールド定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
}

/// 列挙体のバリアント定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<EnumVariantPayload>,
}

/// 列挙体バリアントのデータ型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariantPayload {
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

/// 型環境（型推論・型チェック時に使用）
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    // 型名からTypeIdへのマッピング
    type_map: HashMap<String, TypeId>,
    
    // TypeIdから型定義へのマッピング
    types: HashMap<TypeId, Type>,
    
    // シンボルから型へのマッピング
    symbol_types: HashMap<SymbolId, TypeId>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        let mut env = Self {
            type_map: HashMap::new(),
            types: HashMap::new(),
            symbol_types: HashMap::new(),
        };
        
        // 組み込み型をあらかじめ登録
        env.register_builtin_type("unit", Type::unit());
        env.register_builtin_type("bool", Type::bool());
        env.register_builtin_type("int", Type::int());
        env.register_builtin_type("float", Type::float());
        env.register_builtin_type("char", Type::char());
        env.register_builtin_type("string", Type::string());
        
        env
    }
    
    fn register_builtin_type(&mut self, name: &str, ty: Type) {
        let id = ty.id;
        self.type_map.insert(name.to_string(), id);
        self.types.insert(id, ty);
    }
    
    pub fn register_type(&mut self, name: String, ty: Type) -> TypeId {
        let id = ty.id;
        self.type_map.insert(name, id);
        self.types.insert(id, ty.clone());
        id
    }
    
    pub fn get_type_by_name(&self, name: &str) -> Option<&Type> {
        self.type_map.get(name).and_then(|id| self.types.get(id))
    }
    
    pub fn get_type_by_id(&self, id: TypeId) -> Option<&Type> {
        self.types.get(&id)
    }
    
    pub fn set_symbol_type(&mut self, symbol: SymbolId, type_id: TypeId) {
        self.symbol_types.insert(symbol, type_id);
    }
    
    pub fn get_symbol_type(&self, symbol: SymbolId) -> Option<&Type> {
        self.symbol_types.get(&symbol).and_then(|id| self.types.get(id))
    }
} 