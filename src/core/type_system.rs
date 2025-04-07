// 型システム定義のためのDSL
// Eidosで世界一の汎用言語の型システムを定義するための基盤

use std::collections::{HashMap, HashSet};
use crate::error::ErrorDiagnostic;
use crate::core::language_def::{TypeSystem as LangTypeSystem, TypeDefinition};

/// 型の種類
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// プリミティブ型
    Primitive(PrimitiveType),
    
    /// 名前付き型（ユーザー定義型）
    Named(String),
    
    /// ジェネリック型
    Generic {
        /// 基底型
        base: Box<Type>,
        /// 型引数
        args: Vec<Type>,
    },
    
    /// 関数型
    Function {
        /// パラメータ型
        params: Vec<Type>,
        /// 戻り値型
        return_type: Box<Type>,
        /// 副作用情報
        effects: Option<EffectSet>,
    },
    
    /// タプル型
    Tuple(Vec<Type>),
    
    /// 配列型
    Array {
        /// 要素型
        element: Box<Type>,
        /// サイズ（Noneは動的サイズ）
        size: Option<usize>,
    },
    
    /// ポインタ型
    Pointer {
        /// 指す先の型
        pointee: Box<Type>,
        /// 変更可能性
        mutable: bool,
    },
    
    /// 参照型
    Reference {
        /// 参照先の型
        referenced: Box<Type>,
        /// 変更可能性
        mutable: bool,
        /// ライフタイム情報
        lifetime: Option<Lifetime>,
    },
    
    /// オプション型
    Option(Box<Type>),
    
    /// 結果型
    Result {
        /// 成功型
        ok: Box<Type>,
        /// エラー型
        err: Box<Type>,
    },
    
    /// 代数的データ型（ADT）
    AlgebraicDataType {
        /// バリアント
        variants: Vec<(String, Vec<Type>)>,
        /// タグの有無
        tagged: bool,
    },
    
    /// インターセクション型（すべての型の特性を持つ）
    Intersection(Vec<Type>),
    
    /// ユニオン型（いずれかの型である）
    Union(Vec<Type>),
    
    /// 高階型
    HigherKinded {
        /// 型コンストラクタ
        constructor: String,
        /// 種
        kind: Kind,
    },
    
    /// 依存型
    Dependent {
        /// 依存変数名
        param_name: String,
        /// 型の本体
        body: Box<Type>,
    },
    
    /// 存在型
    Existential {
        /// 型変数名
        var_name: String,
        /// 制約
        constraint: Box<TypeConstraint>,
    },
    
    /// 自己参照型
    Self_,
    
    /// 型変数
    TypeVar(String),
    
    /// ボトム型（すべての型のサブタイプ）
    Bottom,
    
    /// トップ型（すべての型のスーパータイプ）
    Top,
}

/// プリミティブ型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// 整数型（ビット幅と符号付き）
    Integer { bits: u8, signed: bool },
    
    /// 浮動小数点型（ビット幅）
    Float(u8),
    
    /// 文字型
    Char,
    
    /// 真偽値型
    Boolean,
    
    /// 文字列型
    String,
    
    /// ユニット型（空タプル）
    Unit,
    
    /// 不定型（計算が終了しない）
    Never,
    
    /// 動的型
    Dynamic,
}

/// 型の種（カインド）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// 値型
    Type,
    
    /// 型からから型への関数（ジェネリック型）
    Arrow(Box<Kind>, Box<Kind>),
    
    /// 列挙型の種
    Enum,
    
    /// レコード（構造体）の種
    Record,
    
    /// 型レベル整数
    Nat,
    
    /// 型レベル自然数の上限
    NatUpperBound(usize),
}

/// ライフタイム
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lifetime {
    /// 名前付きライフタイム
    Named(String),
    
    /// 静的ライフタイム
    Static,
    
    /// 無名ライフタイム
    Anonymous(usize),
    
    /// ライフタイム変数
    Variable(String),
}

/// 効果セット
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSet {
    /// IO効果
    pub io: bool,
    
    /// 状態変更効果
    pub state: bool,
    
    /// 例外効果
    pub exception: bool,
    
    /// 非局所的制御フロー
    pub nonlocal_control_flow: bool,
    
    /// メモリ割り当て効果
    pub allocation: bool,
    
    /// 並行性効果
    pub concurrency: bool,
    
    /// 名前付きカスタム効果
    pub custom: HashSet<String>,
}

/// 型制約
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// サブタイプ制約
    Subtype {
        /// 部分型
        sub: Type,
        /// 親型
        super_: Type,
    },
    
    /// 型等価性制約
    Equals(Type, Type),
    
    /// トレイト（インターフェース）境界
    HasTrait {
        /// 型
        ty: Type,
        /// トレイト名
        trait_name: String,
        /// トレイト型引数
        trait_args: Vec<Type>,
    },
    
    /// 構造的制約（メソッド/フィールドの存在）
    HasMember {
        /// 型
        ty: Type,
        /// メンバー名
        member_name: String,
        /// メンバー型
        member_type: Type,
    },
    
    /// 型演算の結果
    TypeOperator {
        /// 演算子名
        operator: String,
        /// オペランド
        operands: Vec<Type>,
        /// 結果型
        result: Type,
    },
    
    /// 制約の論理積
    Conjunction(Vec<TypeConstraint>),
    
    /// 制約の論理和
    Disjunction(Vec<TypeConstraint>),
    
    /// 制約の否定
    Negation(Box<TypeConstraint>),
    
    /// 制約の含意
    Implication(Box<TypeConstraint>, Box<TypeConstraint>),
    
    /// 依存型制約
    Refinement {
        /// 基底型
        base: Type,
        /// 述語
        predicate: String,
    },
}

/// 型推論アルゴリズム
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceAlgorithm {
    /// ヒンドリー・ミルナー
    HindleyMilner,
    
    /// 制約ベース
    ConstraintBased,
    
    /// ローカル型推論
    Local,
    
    /// 畳み込み
    Bidirectional,
    
    /// カリー＝ハワード対応に基づく
    CurryHoward,
    
    /// 型クラス解決ベース
    TypeClassResolution,
    
    /// カスタム
    Custom(String),
}

/// 型変性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variance {
    /// 共変（サブタイプ関係を保存）
    Covariant,
    
    /// 反変（サブタイプ関係を反転）
    Contravariant,
    
    /// 不変（型の同一性が必要）
    Invariant,
    
    /// 双変（共変かつ反変）
    Bivariant,
}

/// 型システム実装
#[derive(Debug, Clone)]
pub struct TypeSystemImpl {
    /// 基本型セット
    pub base_types: HashSet<Type>,
    
    /// 型エイリアス
    pub type_aliases: HashMap<String, Type>,
    
    /// 型コンストラクタ
    pub type_constructors: HashMap<String, Kind>,
    
    /// トレイト（インターフェース）定義
    pub traits: HashMap<String, TraitDefinition>,
    
    /// 特殊型演算子
    pub type_operators: HashMap<String, TypeOperator>,
    
    /// 型推論アルゴリズム
    pub inference_algorithm: InferenceAlgorithm,
    
    /// 型付けルール
    pub typing_rules: Vec<TypingRule>,
    
    /// 型変性ルール
    pub variance_rules: HashMap<String, Vec<Variance>>,
    
    /// サブタイプ関係
    pub subtyping_relationships: Vec<(Type, Type)>,
    
    /// 型バインディングスコープ
    pub type_binding_scopes: Vec<TypeBindingScope>,
    
    /// 過負荷解決戦略
    pub overload_resolution: OverloadResolutionStrategy,
    
    /// 型パラメータのデフォルト値
    pub default_type_parameters: HashMap<String, Type>,
    
    /// 型安全性保証
    pub safety_guarantees: TypeSafetyGuarantees,
}

/// トレイト（インターフェース）定義
#[derive(Debug, Clone)]
pub struct TraitDefinition {
    /// トレイト名
    pub name: String,
    
    /// 型パラメータ
    pub type_parameters: Vec<TypeParameter>,
    
    /// 関連型
    pub associated_types: Vec<String>,
    
    /// メソッドシグネチャ
    pub method_signatures: Vec<MethodSignature>,
    
    /// 関連定数
    pub associated_constants: Vec<(String, Type)>,
    
    /// スーパートレイト
    pub super_traits: Vec<String>,
    
    /// デフォルト実装
    pub default_implementations: HashMap<String, String>,
}

/// 型パラメータ
#[derive(Debug, Clone)]
pub struct TypeParameter {
    /// パラメータ名
    pub name: String,
    
    /// 制約
    pub constraints: Vec<TypeConstraint>,
    
    /// デフォルト値
    pub default_value: Option<Type>,
    
    /// 種
    pub kind: Kind,
    
    /// 変性
    pub variance: Variance,
}

/// メソッドシグネチャ
#[derive(Debug, Clone)]
pub struct MethodSignature {
    /// メソッド名
    pub name: String,
    
    /// 型パラメータ
    pub type_parameters: Vec<TypeParameter>,
    
    /// パラメータ型
    pub parameter_types: Vec<Type>,
    
    /// 戻り値型
    pub return_type: Type,
    
    /// 効果
    pub effects: Option<EffectSet>,
    
    /// 事前条件
    pub preconditions: Vec<String>,
    
    /// 事後条件
    pub postconditions: Vec<String>,
}

/// 型演算子
#[derive(Debug, Clone)]
pub struct TypeOperator {
    /// 演算子名
    pub name: String,
    
    /// 優先順位
    pub precedence: u8,
    
    /// 結合性（左結合 or 右結合）
    pub associativity: Associativity,
    
    /// オペランド数
    pub arity: usize,
    
    /// 実装関数（評価方法）
    pub implementation: String,
}

/// 結合性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// 左結合
    Left,
    
    /// 右結合
    Right,
    
    /// 非結合的
    None,
}

/// 型付けルール
#[derive(Debug, Clone)]
pub struct TypingRule {
    /// ルール名
    pub name: String,
    
    /// 前提（条件）
    pub premises: Vec<TypeJudgment>,
    
    /// 結論
    pub conclusion: TypeJudgment,
    
    /// 説明
    pub description: String,
}

/// 型判断
#[derive(Debug, Clone)]
pub struct TypeJudgment {
    /// 型環境
    pub environment: String,
    
    /// 式
    pub expression: String,
    
    /// 型
    pub type_: Type,
    
    /// 効果
    pub effects: Option<EffectSet>,
}

/// 型バインディングスコープ
#[derive(Debug, Clone)]
pub struct TypeBindingScope {
    /// スコープ名
    pub name: String,
    
    /// 親スコープ
    pub parent: Option<String>,
    
    /// バインディング
    pub bindings: HashMap<String, Type>,
}

/// 過負荷解決戦略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadResolutionStrategy {
    /// 最も特殊な実装
    MostSpecific,
    
    /// 優先度ベース
    PriorityBased,
    
    /// 多重ディスパッチ
    MultipleDispatch,
    
    /// コンテキスト依存
    ContextDependent,
    
    /// 静的
    Static,
    
    /// 動的
    Dynamic,
    
    /// カスタム
    Custom(String),
}

/// 型安全性保証
#[derive(Debug, Clone)]
pub struct TypeSafetyGuarantees {
    /// NULL安全性
    pub null_safety: NullSafety,
    
    /// メモリ安全性
    pub memory_safety: MemorySafety,
    
    /// スレッド安全性
    pub thread_safety: ThreadSafety,
    
    /// 例外安全性
    pub exception_safety: ExceptionSafety,
    
    /// 実行時型チェック
    pub runtime_type_checking: bool,
    
    /// コンパイル時型チェック
    pub compile_time_type_checking: bool,
    
    /// 型保存性
    pub type_preservation: bool,
    
    /// 進行保証
    pub progress_guarantee: bool,
}

/// NULL安全性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullSafety {
    /// なし
    None,
    
    /// 部分的
    Partial,
    
    /// 厳格
    Strict,
}

/// メモリ安全性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySafety {
    /// 制約なし
    Unconstrained,
    
    /// 実行時チェック
    RuntimeChecked,
    
    /// コンパイル時検証
    CompileTimeVerified,
    
    /// 型システムで強制
    TypeSystemEnforced,
}

/// スレッド安全性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadSafety {
    /// なし
    None,
    
    /// 部分的
    Partial,
    
    /// 型強制
    TypeEnforced,
    
    /// 排他制御
    Synchronized,
}

/// 例外安全性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionSafety {
    /// 保証なし
    None,
    
    /// 基本保証
    Basic,
    
    /// 強保証
    Strong,
    
    /// 例外なし
    NoThrow,
    
    /// コンパイル時チェック
    CompileTimeChecked,
}

impl TypeSystemImpl {
    /// 新しい型システムを作成
    pub fn new(inference_algorithm: InferenceAlgorithm) -> Self {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Strict,
            memory_safety: MemorySafety::TypeSystemEnforced,
            thread_safety: ThreadSafety::TypeEnforced,
            exception_safety: ExceptionSafety::CompileTimeChecked,
            runtime_type_checking: false,
            compile_time_type_checking: true,
            type_preservation: true,
            progress_guarantee: true,
        };
        
        TypeSystemImpl {
            base_types: HashSet::new(),
            type_aliases: HashMap::new(),
            type_constructors: HashMap::new(),
            traits: HashMap::new(),
            type_operators: HashMap::new(),
            inference_algorithm,
            typing_rules: Vec::new(),
            variance_rules: HashMap::new(),
            subtyping_relationships: Vec::new(),
            type_binding_scopes: Vec::new(),
            overload_resolution: OverloadResolutionStrategy::MostSpecific,
            default_type_parameters: HashMap::new(),
            safety_guarantees,
        }
    }
    
    /// 基本型を追加
    pub fn add_base_type(mut self, ty: Type) -> Self {
        self.base_types.insert(ty);
        self
    }
    
    /// 型エイリアスを追加
    pub fn add_type_alias(mut self, name: &str, ty: Type) -> Self {
        self.type_aliases.insert(name.to_string(), ty);
        self
    }
    
    /// 型コンストラクタを追加
    pub fn add_type_constructor(mut self, name: &str, kind: Kind) -> Self {
        self.type_constructors.insert(name.to_string(), kind);
        self
    }
    
    /// トレイトを追加
    pub fn add_trait(mut self, trait_def: TraitDefinition) -> Self {
        self.traits.insert(trait_def.name.clone(), trait_def);
        self
    }
    
    /// 型演算子を追加
    pub fn add_type_operator(mut self, operator: TypeOperator) -> Self {
        self.type_operators.insert(operator.name.clone(), operator);
        self
    }
    
    /// 型付けルールを追加
    pub fn add_typing_rule(mut self, rule: TypingRule) -> Self {
        self.typing_rules.push(rule);
        self
    }
    
    /// 型変性ルールを追加
    pub fn add_variance_rule(mut self, type_name: &str, variances: Vec<Variance>) -> Self {
        self.variance_rules.insert(type_name.to_string(), variances);
        self
    }
    
    /// サブタイプ関係を追加
    pub fn add_subtyping_relationship(mut self, sub: Type, super_: Type) -> Self {
        self.subtyping_relationships.push((sub, super_));
        self
    }
    
    /// 型バインディングスコープを追加
    pub fn add_type_binding_scope(mut self, scope: TypeBindingScope) -> Self {
        self.type_binding_scopes.push(scope);
        self
    }
    
    /// 過負荷解決戦略を設定
    pub fn with_overload_resolution(mut self, strategy: OverloadResolutionStrategy) -> Self {
        self.overload_resolution = strategy;
        self
    }
    
    /// デフォルト型パラメータを追加
    pub fn add_default_type_parameter(mut self, param_name: &str, default_value: Type) -> Self {
        self.default_type_parameters.insert(param_name.to_string(), default_value);
        self
    }
    
    /// 型安全性保証を設定
    pub fn with_safety_guarantees(mut self, guarantees: TypeSafetyGuarantees) -> Self {
        self.safety_guarantees = guarantees;
        self
    }
    
    /// 型システムを検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // 型システムの検証ロジックを実装...
        // - 型のサイクル検出
        // - トレイト整合性検証
        // - サブタイプ関係の推移性チェック
        // など
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 言語定義用のTypeSystem構造体に変換
    pub fn to_language_type_system(&self) -> LangTypeSystem {
        // 実際の実装では対応する構造体に変換
        LangTypeSystem::default()
    }
}

/// 型システムDSLの便利なコンストラクタ
pub mod dsl {
    use super::*;
    
    /// 静的型付け型システムを作成
    pub fn static_typing() -> TypeSystemImpl {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Strict,
            memory_safety: MemorySafety::TypeSystemEnforced,
            thread_safety: ThreadSafety::TypeEnforced,
            exception_safety: ExceptionSafety::CompileTimeChecked,
            runtime_type_checking: false,
            compile_time_type_checking: true,
            type_preservation: true,
            progress_guarantee: true,
        };
        
        let ts = TypeSystemImpl::new(InferenceAlgorithm::HindleyMilner)
            .with_safety_guarantees(safety_guarantees);
        
        // 基本型を追加
        let base_types = vec![
            Type::Primitive(PrimitiveType::Integer { bits: 32, signed: true }),
            Type::Primitive(PrimitiveType::Integer { bits: 32, signed: false }),
            Type::Primitive(PrimitiveType::Integer { bits: 64, signed: true }),
            Type::Primitive(PrimitiveType::Integer { bits: 64, signed: false }),
            Type::Primitive(PrimitiveType::Float(32)),
            Type::Primitive(PrimitiveType::Float(64)),
            Type::Primitive(PrimitiveType::Boolean),
            Type::Primitive(PrimitiveType::Char),
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Unit),
            Type::Primitive(PrimitiveType::Never),
        ];
        
        let mut result = ts;
        for ty in base_types {
            result = result.add_base_type(ty);
        }
        
        result
    }
    
    /// 動的型付け型システムを作成
    pub fn dynamic_typing() -> TypeSystemImpl {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Partial,
            memory_safety: MemorySafety::RuntimeChecked,
            thread_safety: ThreadSafety::None,
            exception_safety: ExceptionSafety::Basic,
            runtime_type_checking: true,
            compile_time_type_checking: false,
            type_preservation: false,
            progress_guarantee: false,
        };
        
        TypeSystemImpl::new(InferenceAlgorithm::Local)
            .with_safety_guarantees(safety_guarantees)
            .add_base_type(Type::Primitive(PrimitiveType::Dynamic))
    }
    
    /// 漸進的型付け型システムを作成
    pub fn gradual_typing() -> TypeSystemImpl {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Partial,
            memory_safety: MemorySafety::RuntimeChecked,
            thread_safety: ThreadSafety::Partial,
            exception_safety: ExceptionSafety::Basic,
            runtime_type_checking: true,
            compile_time_type_checking: true,
            type_preservation: true,
            progress_guarantee: true,
        };
        
        let ts = TypeSystemImpl::new(InferenceAlgorithm::Bidirectional)
            .with_safety_guarantees(safety_guarantees);
        
        // 基本型を追加
        let base_types = vec![
            Type::Primitive(PrimitiveType::Integer { bits: 32, signed: true }),
            Type::Primitive(PrimitiveType::Integer { bits: 64, signed: true }),
            Type::Primitive(PrimitiveType::Float(64)),
            Type::Primitive(PrimitiveType::Boolean),
            Type::Primitive(PrimitiveType::String),
            Type::Primitive(PrimitiveType::Unit),
            Type::Primitive(PrimitiveType::Dynamic),
        ];
        
        let mut result = ts;
        for ty in base_types {
            result = result.add_base_type(ty);
        }
        
        result
    }
    
    /// 依存型システムを作成
    pub fn dependent_typing() -> TypeSystemImpl {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Strict,
            memory_safety: MemorySafety::TypeSystemEnforced,
            thread_safety: ThreadSafety::TypeEnforced,
            exception_safety: ExceptionSafety::CompileTimeChecked,
            runtime_type_checking: false,
            compile_time_type_checking: true,
            type_preservation: true,
            progress_guarantee: true,
        };
        
        TypeSystemImpl::new(InferenceAlgorithm::CurryHoward)
            .with_safety_guarantees(safety_guarantees)
    }
    
    /// リッチな型システムを作成
    pub fn rich_typing() -> TypeSystemImpl {
        let safety_guarantees = TypeSafetyGuarantees {
            null_safety: NullSafety::Strict,
            memory_safety: MemorySafety::TypeSystemEnforced,
            thread_safety: ThreadSafety::TypeEnforced,
            exception_safety: ExceptionSafety::CompileTimeChecked,
            runtime_type_checking: true,
            compile_time_type_checking: true,
            type_preservation: true,
            progress_guarantee: true,
        };
        
        TypeSystemImpl::new(InferenceAlgorithm::ConstraintBased)
            .with_safety_guarantees(safety_guarantees)
            .with_overload_resolution(OverloadResolutionStrategy::MultipleDispatch)
    }
}

/// 型システムDSLのマクロ
#[macro_export]
macro_rules! type_system {
    // 型システム定義全体
    ($system_type:ident {
        $($content:tt)*
    }) => {
        {
            let mut ts = match stringify!($system_type) {
                "static" => $crate::core::type_system::dsl::static_typing(),
                "dynamic" => $crate::core::type_system::dsl::dynamic_typing(),
                "gradual" => $crate::core::type_system::dsl::gradual_typing(),
                "dependent" => $crate::core::type_system::dsl::dependent_typing(),
                "rich" => $crate::core::type_system::dsl::rich_typing(),
                _ => $crate::core::type_system::dsl::static_typing(),
            };
            
            $(
                $crate::type_system_item!(ts, $content);
            )*
            
            ts
        }
    };
}

/// 型システム項目のマクロヘルパー
#[macro_export]
macro_rules! type_system_item {
    // 基本型追加
    ($ts:ident, base_type $kind:ident $([ $($param:expr),* ])? ;) => {
        {
            $(
                let ty = match stringify!($kind) {
                    "int" => Type::Primitive(PrimitiveType::Integer { bits: $($param as u8),*, signed: true }),
                    "uint" => Type::Primitive(PrimitiveType::Integer { bits: $($param as u8),*, signed: false }),
                    "float" => Type::Primitive(PrimitiveType::Float($($param as u8),*)),
                    "bool" => Type::Primitive(PrimitiveType::Boolean),
                    "char" => Type::Primitive(PrimitiveType::Char),
                    "string" => Type::Primitive(PrimitiveType::String),
                    "unit" => Type::Primitive(PrimitiveType::Unit),
                    "never" => Type::Primitive(PrimitiveType::Never),
                    "dynamic" => Type::Primitive(PrimitiveType::Dynamic),
                    _ => Type::Named(stringify!($kind).to_string()),
                };
            )*
            $ts = $ts.add_base_type(ty);
        }
    };
    
    // 型エイリアス追加
    ($ts:ident, alias $name:ident = $ty_expr:expr;) => {
        {
            $ts = $ts.add_type_alias(stringify!($name), $ty_expr);
        }
    };
    
    // サブタイプ関係追加
    ($ts:ident, subtype $sub:expr <: $super:expr;) => {
        {
            $ts = $ts.add_subtyping_relationship($sub, $super);
        }
    };
    
    // 安全性保証設定
    ($ts:ident, safety {
        null = $null:ident;
        memory = $memory:ident;
        thread = $thread:ident;
        exception = $exception:ident;
        $(runtime_checks = $runtime:expr;)?
        $(compile_time_checks = $compile:expr;)?
    }) => {
        {
            let safety = TypeSafetyGuarantees {
                null_safety: NullSafety::$null,
                memory_safety: MemorySafety::$memory,
                thread_safety: ThreadSafety::$thread,
                exception_safety: ExceptionSafety::$exception,
                runtime_type_checking: $($runtime)?,
                compile_time_type_checking: $($compile)?,
                type_preservation: true,
                progress_guarantee: true,
            };
            $ts = $ts.with_safety_guarantees(safety);
        }
    };
    
    // 型推論アルゴリズム設定
    ($ts:ident, inference = $algo:ident;) => {
        {
            $ts = TypeSystemImpl::new(InferenceAlgorithm::$algo);
        }
    };
} 