// 型システム定義のためのDSL
// Eidosで世界一の汎用言語の型システムを定義するための基盤

use std::collections::{HashMap, HashSet};
use crate::error::ErrorDiagnostic;
use crate::core::language_def::{TypeSystem, TypeDefinition, TypeParameter, TypeRule, TypeConstraint, TypeJudgment, InferenceRule, SubtypeRelation, ConversionRule};

/// 基本型の定義
#[derive(Debug, Clone)]
pub enum BaseType {
    /// 整数型
    Integer {
        /// ビット幅
        bit_width: usize,
        /// 符号付きか
        signed: bool,
    },
    
    /// 浮動小数点型
    Float {
        /// ビット幅
        bit_width: usize,
    },
    
    /// 文字型
    Char {
        /// Unicode（UTF-8/16/32）か
        unicode: bool,
    },
    
    /// 真偽値型
    Boolean,
    
    /// 単位型（void など）
    Unit,
    
    /// ボトム型（never など）
    Bottom,
    
    /// トップ型（any など）
    Top,
    
    /// アドレス型（ポインタなど）
    Address {
        /// アドレス空間
        address_space: usize,
    },
    
    /// 列挙型
    Enum {
        /// 列挙値
        values: Vec<String>,
    },
}

/// 型の定義
#[derive(Debug, Clone)]
pub enum Type {
    /// 名前付き型
    Named(String),
    
    /// 配列型
    Array(Box<Type>, Option<usize>),
    
    /// タプル型
    Tuple(Vec<Type>),
    
    /// 関数型
    Function(Vec<Type>, Box<Type>),
    
    /// ジェネリック型
    Generic(String, Vec<Type>),
    
    /// 共用体型
    Union(Vec<Type>),
    
    /// 交差型
    Intersection(Vec<Type>),
    
    /// 参照型
    Reference(Box<Type>, bool),
    
    /// 型パラメータ
    TypeParam(String),
    
    /// 依存型
    Dependent(String, Box<Type>),
    
    /// 存在型
    Existential(String, Box<Type>),
    
    /// 線形型
    Linear(Box<Type>),
    
    /// 値依存型
    ValueDependent(String, String, Box<Type>),
}

impl Type {
    /// 名前付き型を作成
    pub fn named(name: &str) -> Self {
        Type::Named(name.to_string())
    }
    
    /// 配列型を作成
    pub fn array(element_type: Type, size: Option<usize>) -> Self {
        Type::Array(Box::new(element_type), size)
    }
    
    /// タプル型を作成
    pub fn tuple(types: Vec<Type>) -> Self {
        Type::Tuple(types)
    }
    
    /// 関数型を作成
    pub fn function(param_types: Vec<Type>, return_type: Type) -> Self {
        Type::Function(param_types, Box::new(return_type))
    }
    
    /// ジェネリック型を作成
    pub fn generic(name: &str, type_args: Vec<Type>) -> Self {
        Type::Generic(name.to_string(), type_args)
    }
    
    /// 共用体型を作成
    pub fn union(types: Vec<Type>) -> Self {
        Type::Union(types)
    }
    
    /// 交差型を作成
    pub fn intersection(types: Vec<Type>) -> Self {
        Type::Intersection(types)
    }
    
    /// 参照型を作成
    pub fn reference(target_type: Type, mutable: bool) -> Self {
        Type::Reference(Box::new(target_type), mutable)
    }
    
    /// 型パラメータを作成
    pub fn type_param(name: &str) -> Self {
        Type::TypeParam(name.to_string())
    }
    
    /// 依存型を作成
    pub fn dependent(param_name: &str, body_type: Type) -> Self {
        Type::Dependent(param_name.to_string(), Box::new(body_type))
    }
    
    /// 存在型を作成
    pub fn existential(param_name: &str, body_type: Type) -> Self {
        Type::Existential(param_name.to_string(), Box::new(body_type))
    }
    
    /// 線形型を作成
    pub fn linear(inner_type: Type) -> Self {
        Type::Linear(Box::new(inner_type))
    }
    
    /// 値依存型を作成
    pub fn value_dependent(var_name: &str, type_name: &str, body_type: Type) -> Self {
        Type::ValueDependent(var_name.to_string(), type_name.to_string(), Box::new(body_type))
    }
    
    /// 型を文字列に変換
    pub fn to_string(&self) -> String {
        match self {
            Type::Named(name) => name.clone(),
            Type::Array(elem_type, Some(size)) => format!("[{}; {}]", elem_type.to_string(), size),
            Type::Array(elem_type, None) => format!("[{}]", elem_type.to_string()),
            Type::Tuple(types) => {
                let types_str = types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", types_str)
            },
            Type::Function(param_types, return_type) => {
                let param_str = param_types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", param_str, return_type.to_string())
            },
            Type::Generic(name, type_args) => {
                let args_str = type_args.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args_str)
            },
            Type::Union(types) => {
                let types_str = types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("({})", types_str)
            },
            Type::Intersection(types) => {
                let types_str = types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" & ");
                format!("({})", types_str)
            },
            Type::Reference(target_type, mutable) => {
                if *mutable {
                    format!("&mut {}", target_type.to_string())
                } else {
                    format!("&{}", target_type.to_string())
                }
            },
            Type::TypeParam(name) => format!("'{}", name),
            Type::Dependent(param_name, body_type) => {
                format!("∀{}.{}", param_name, body_type.to_string())
            },
            Type::Existential(param_name, body_type) => {
                format!("∃{}.{}", param_name, body_type.to_string())
            },
            Type::Linear(inner_type) => {
                format!("linear {}", inner_type.to_string())
            },
            Type::ValueDependent(var_name, type_name, body_type) => {
                format!("Π{}:{}.{}", var_name, type_name, body_type.to_string())
            },
        }
    }
    
    /// 型が等しいかチェック
    pub fn equals(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Named(name1), Type::Named(name2)) => name1 == name2,
            (Type::Array(elem1, size1), Type::Array(elem2, size2)) => {
                elem1.equals(elem2) && size1 == size2
            },
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                types1.iter().zip(types2.iter()).all(|(t1, t2)| t1.equals(t2))
            },
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return false;
                }
                params1.iter().zip(params2.iter()).all(|(p1, p2)| p1.equals(p2)) && ret1.equals(ret2)
            },
            (Type::Generic(name1, args1), Type::Generic(name2, args2)) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return false;
                }
                args1.iter().zip(args2.iter()).all(|(a1, a2)| a1.equals(a2))
            },
            (Type::Union(types1), Type::Union(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                // 順序に依存しないように、各型が他方の集合に存在するかを確認
                types1.iter().all(|t1| types2.iter().any(|t2| t1.equals(t2))) &&
                types2.iter().all(|t2| types1.iter().any(|t1| t1.equals(t2)))
            },
            (Type::Intersection(types1), Type::Intersection(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                // 順序に依存しないように、各型が他方の集合に存在するかを確認
                types1.iter().all(|t1| types2.iter().any(|t2| t1.equals(t2))) &&
                types2.iter().all(|t2| types1.iter().any(|t1| t1.equals(t2)))
            },
            (Type::Reference(target1, mutable1), Type::Reference(target2, mutable2)) => {
                mutable1 == mutable2 && target1.equals(target2)
            },
            (Type::TypeParam(name1), Type::TypeParam(name2)) => name1 == name2,
            (Type::Dependent(param1, body1), Type::Dependent(param2, body2)) => {
                param1 == param2 && body1.equals(body2)
            },
            (Type::Existential(param1, body1), Type::Existential(param2, body2)) => {
                param1 == param2 && body1.equals(body2)
            },
            (Type::Linear(inner1), Type::Linear(inner2)) => inner1.equals(inner2),
            (Type::ValueDependent(var1, type1, body1), Type::ValueDependent(var2, type2, body2)) => {
                var1 == var2 && type1 == type2 && body1.equals(body2)
            },
            // 異なる型の場合は等しくない
            _ => false,
        }
    }
}

/// 型制約の種類
#[derive(Debug, Clone)]
pub enum TypeConstraintKind {
    /// 等しい
    Equal,
    
    /// サブタイプ
    Subtype,
    
    /// 異なる
    Different,
    
    /// 型クラス制約
    TypeClass(String),
    
    /// 測定制約
    Sized,
    
    /// コピー可能制約
    Copyable,
    
    /// 所有権制約
    Owned,
    
    /// 借用制約
    Borrowed,
    
    /// 線形制約
    Linear,
    
    /// カスタム制約
    Custom(String),
}

/// 型制約の実装
#[derive(Debug, Clone)]
pub struct TypeConstraintImpl {
    /// 左辺の型
    pub left: Type,
    
    /// 制約の種類
    pub kind: TypeConstraintKind,
    
    /// 右辺の型（オプション）
    pub right: Option<Type>,
}

impl TypeConstraintImpl {
    /// 等値制約を作成
    pub fn equal(left: Type, right: Type) -> Self {
        TypeConstraintImpl {
            left,
            kind: TypeConstraintKind::Equal,
            right: Some(right),
        }
    }
    
    /// サブタイプ制約を作成
    pub fn subtype(left: Type, right: Type) -> Self {
        TypeConstraintImpl {
            left,
            kind: TypeConstraintKind::Subtype,
            right: Some(right),
        }
    }
    
    /// 型クラス制約を作成
    pub fn type_class(type_: Type, class_name: &str) -> Self {
        TypeConstraintImpl {
            left: type_,
            kind: TypeConstraintKind::TypeClass(class_name.to_string()),
            right: None,
        }
    }
    
    /// 型制約を文字列に変換
    pub fn to_string(&self) -> String {
        match &self.kind {
            TypeConstraintKind::Equal => {
                format!("{} = {}", self.left.to_string(), self.right.as_ref().map_or("?".to_string(), |r| r.to_string()))
            },
            TypeConstraintKind::Subtype => {
                format!("{} <: {}", self.left.to_string(), self.right.as_ref().map_or("?".to_string(), |r| r.to_string()))
            },
            TypeConstraintKind::Different => {
                format!("{} != {}", self.left.to_string(), self.right.as_ref().map_or("?".to_string(), |r| r.to_string()))
            },
            TypeConstraintKind::TypeClass(class_name) => {
                format!("{}: {}", self.left.to_string(), class_name)
            },
            TypeConstraintKind::Sized => {
                format!("Sized({})", self.left.to_string())
            },
            TypeConstraintKind::Copyable => {
                format!("Copy({})", self.left.to_string())
            },
            TypeConstraintKind::Owned => {
                format!("Owned({})", self.left.to_string())
            },
            TypeConstraintKind::Borrowed => {
                format!("Borrowed({})", self.left.to_string())
            },
            TypeConstraintKind::Linear => {
                format!("Linear({})", self.left.to_string())
            },
            TypeConstraintKind::Custom(name) => {
                format!("{}({})", name, self.left.to_string())
            },
        }
    }
}

/// 型ルールの実装
#[derive(Debug, Clone)]
pub struct TypeRuleImpl {
    /// ルール名
    pub name: String,
    
    /// 前提条件
    pub premises: Vec<TypeJudgmentImpl>,
    
    /// 結論
    pub conclusion: TypeJudgmentImpl,
}

impl TypeRuleImpl {
    /// 型ルールを文字列に変換
    pub fn to_string(&self) -> String {
        let premises_str = if self.premises.is_empty() {
            "∅".to_string()
        } else {
            self.premises.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        format!("{} ⊢ {} [{}]", premises_str, self.conclusion.to_string(), self.name)
    }
    
    /// 型ルールを検証
    pub fn validate(&self, base_types: &HashMap<String, BaseType>) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // ルール名の検証
        if self.name.is_empty() {
            errors.push(ErrorDiagnostic::new(
                "型ルール名が空です",
                "型ルールには有効な名前を設定してください",
                None,
                None,
            ));
        }
        
        // 前提条件の検証
        for (i, premise) in self.premises.iter().enumerate() {
            if premise.expression.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    format!("前提条件 #{} の式が空です", i + 1),
                    "前提条件には有効な式を設定してください",
                    None,
                    None,
                ));
            }
            
            // 型の検証
            self.validate_type(&premise.type_, base_types, &mut errors, &format!("前提条件 #{}", i + 1));
        }
        
        // 結論の検証
        if self.conclusion.expression.is_empty() {
            errors.push(ErrorDiagnostic::new(
                "結論の式が空です",
                "結論には有効な式を設定してください",
                None,
                None,
            ));
        }
        
        // 結論の型の検証
        self.validate_type(&self.conclusion.type_, base_types, &mut errors, "結論");
        
        // 前提条件と結論の型の整合性を検証
        self.validate_type_consistency(&mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 型の有効性を検証
    fn validate_type(&self, type_: &Type, base_types: &HashMap<String, BaseType>, errors: &mut Vec<ErrorDiagnostic>, context: &str) {
        match type_ {
            Type::Named(name) => {
                if !base_types.contains_key(name) && !name.starts_with('\'') {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で未定義の型名 '{}' が使用されています", context, name),
                        "定義済みの型名を使用するか、型パラメータとして宣言してください",
                        None,
                        None,
                    ));
                }
            },
            Type::Array(elem_type, size) => {
                if *size == 0 {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で配列サイズが0の型が使用されています", context),
                        "配列サイズは1以上の値を指定してください",
                        None,
                        None,
                    ));
                }
                self.validate_type(elem_type, base_types, errors, context);
            },
            Type::Tuple(types) => {
                for (i, t) in types.iter().enumerate() {
                    self.validate_type(t, base_types, errors, &format!("{}のタプル要素 #{}", context, i + 1));
                }
            },
            Type::Function(param_types, return_type) => {
                for (i, t) in param_types.iter().enumerate() {
                    self.validate_type(t, base_types, errors, &format!("{}の関数パラメータ #{}", context, i + 1));
                }
                self.validate_type(return_type, base_types, errors, &format!("{}の戻り値型", context));
            },
            Type::Generic(name, type_args) => {
                if !base_types.contains_key(name) {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で未定義のジェネリック型名 '{}' が使用されています", context, name),
                        "定義済みのジェネリック型名を使用してください",
                        None,
                        None,
                    ));
                }
                for (i, t) in type_args.iter().enumerate() {
                    self.validate_type(t, base_types, errors, &format!("{}のジェネリック型引数 #{}", context, i + 1));
                }
            },
            Type::Union(types) | Type::Intersection(types) => {
                if types.is_empty() {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で空の合成型（ユニオン/インターセクション）が使用されています", context),
                        "合成型には少なくとも1つの型を含めてください",
                        None,
                        None,
                    ));
                }
                for (i, t) in types.iter().enumerate() {
                    self.validate_type(t, base_types, errors, &format!("{}の合成型要素 #{}", context, i + 1));
                }
            },
            Type::Reference(target_type, _) => {
                self.validate_type(target_type, base_types, errors, &format!("{}の参照対象型", context));
            },
            Type::TypeParam(name) => {
                if name.is_empty() {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で名前のない型パラメータが使用されています", context),
                        "型パラメータには有効な名前を設定してください",
                        None,
                        None,
                    ));
                }
            },
            Type::Dependent(param_name, body_type) | Type::Existential(param_name, body_type) => {
                if param_name.is_empty() {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で名前のない依存型/存在型パラメータが使用されています", context),
                        "型パラメータには有効な名前を設定してください",
                        None,
                        None,
                    ));
                }
                self.validate_type(body_type, base_types, errors, &format!("{}の本体型", context));
            },
            Type::Linear(inner_type) => {
                self.validate_type(inner_type, base_types, errors, &format!("{}の線形型内部", context));
            },
            Type::ValueDependent(var_name, type_name, body_type) => {
                if var_name.is_empty() {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で名前のない値依存型変数が使用されています", context),
                        "値依存型変数には有効な名前を設定してください",
                        None,
                        None,
                    ));
                }
                if type_name.is_empty() {
                    errors.push(ErrorDiagnostic::new(
                        format!("{} で名前のない値依存型の型名が使用されています", context),
                        "値依存型の型名には有効な名前を設定してください",
                        None,
                        None,
                    ));
                }
                self.validate_type(body_type, base_types, errors, &format!("{}の値依存型本体", context));
            },
        }
    }
    
    /// 前提条件と結論の型の整合性を検証
    fn validate_type_consistency(&self, errors: &mut Vec<ErrorDiagnostic>) {
        // 結論で使用されている型変数が前提条件で導入されているか確認
        let mut premise_type_vars = HashSet::new();
        
        // 前提条件から型変数を収集
        for premise in &self.premises {
            self.collect_type_vars(&premise.type_, &mut premise_type_vars);
        }
        
        // 結論の型変数を検証
        let mut conclusion_type_vars = HashSet::new();
        self.collect_type_vars(&self.conclusion.type_, &mut conclusion_type_vars);
        
        for var in &conclusion_type_vars {
            if !premise_type_vars.contains(var) && !var.starts_with('\'') {
                errors.push(ErrorDiagnostic::new(
                    format!("結論で使用されている型変数 '{}' が前提条件で導入されていません", var),
                    "結論で使用する型変数は前提条件で導入するか、型パラメータとして宣言してください",
                    None,
                    None,
                ));
            }
        }
    }
    
    /// 型から型変数を収集
    fn collect_type_vars(&self, type_: &Type, vars: &mut HashSet<String>) {
        match type_ {
            Type::Named(name) => {
                if name.starts_with('\'') {
                    vars.insert(name.clone());
                }
            },
            Type::Array(elem_type, _) => {
                self.collect_type_vars(elem_type, vars);
            },
            Type::Tuple(types) => {
                for t in types {
                    self.collect_type_vars(t, vars);
                }
            },
            Type::Function(param_types, return_type) => {
                for t in param_types {
                    self.collect_type_vars(t, vars);
                }
                self.collect_type_vars(return_type, vars);
            },
            Type::Generic(_, type_args) => {
                for t in type_args {
                    self.collect_type_vars(t, vars);
                }
            },
            Type::Union(types) | Type::Intersection(types) => {
                for t in types {
                    self.collect_type_vars(t, vars);
                }
            },
            Type::Reference(target_type, _) => {
                self.collect_type_vars(target_type, vars);
            },
            Type::TypeParam(name) => {
                vars.insert(name.clone());
            },
            Type::Dependent(_, body_type) | Type::Existential(_, body_type) => {
                self.collect_type_vars(body_type, vars);
            },
            Type::Linear(inner_type) => {
                self.collect_type_vars(inner_type, vars);
            },
            Type::ValueDependent(_, _, body_type) => {
                self.collect_type_vars(body_type, vars);
            },
        }
    }
    
    /// 新しい型ルールを作成
    pub fn new(name: &str, premises: Vec<TypeJudgmentImpl>, conclusion: TypeJudgmentImpl) -> Self {
        TypeRuleImpl {
            name: name.to_string(),
            premises,
            conclusion,
        }
    }
}

/// 型判断の実装
#[derive(Debug, Clone)]
pub struct TypeJudgmentImpl {
    /// 対象の式
    pub expression: String,
    
    /// 型
    pub type_: Type,
}

impl TypeJudgmentImpl {
    /// 型判断を文字列に変換
    pub fn to_string(&self) -> String {
        format!("{}: {}", self.expression, self.type_.to_string())
    }
}

/// 型推論エンジン
pub struct TypeInferenceEngine {
    /// 型環境
    pub type_env: HashMap<String, Type>,
    
    /// 型制約
    pub constraints: Vec<TypeConstraintImpl>,
    
    /// 型ルール
    pub rules: Vec<TypeRuleImpl>,
    
    /// サブタイプ関係
    pub subtype_relations: Vec<SubtypeRelationImpl>,
    
    /// 次に生成する型変数の番号
    pub next_type_var: usize,
}

impl TypeInferenceEngine {
    /// 新しい型推論エンジンを作成
    pub fn new() -> Self {
        TypeInferenceEngine {
            type_env: HashMap::new(),
            constraints: Vec::new(),
            rules: Vec::new(),
            subtype_relations: Vec::new(),
            next_type_var: 0,
        }
    }
    
    /// 型環境に変数を追加
    pub fn add_var(&mut self, name: &str, type_: Type) {
        self.type_env.insert(name.to_string(), type_);
    }
    
    /// 型制約を追加
    pub fn add_constraint(&mut self, constraint: TypeConstraintImpl) {
        self.constraints.push(constraint);
    }
    
    /// 型ルールを追加
    pub fn add_rule(&mut self, rule: TypeRuleImpl) {
        self.rules.push(rule);
    }
    
    /// サブタイプ関係を追加
    pub fn add_subtype_relation(&mut self, relation: SubtypeRelationImpl) {
        self.subtype_relations.push(relation);
    }
    
    /// 新しい型変数を生成
    pub fn fresh_type_var(&mut self) -> Type {
        let var_name = format!("T{}", self.next_type_var);
        self.next_type_var += 1;
        Type::TypeParam(var_name)
    }
    
    /// 型推論を実行
    pub fn infer(&mut self, expr: &str) -> Result<Type, Vec<ErrorDiagnostic>> {
        // 構文解析を行い、AST（抽象構文木）を取得
        let ast = match self.parse_expression(expr) {
            Ok(ast) => ast,
            Err(err) => return Err(vec![err]),
        };
        
        // ASTに基づいて型推論を実行
        let inferred_type = self.infer_ast(&ast)?;
        
        // 制約を解決
        self.solve_constraints()?;
        
        // 推論された型を正規化
        let normalized_type = self.normalize_type(&inferred_type)?;
        
        Ok(normalized_type)
    }
    
    /// 式を解析してASTを生成
    fn parse_expression(&self, expr: &str) -> Result<ExpressionAST, ErrorDiagnostic> {
        // 実際の実装では構文解析ライブラリを使用するか、独自のパーサーを実装
        
        // 簡易的な実装として、いくつかの基本パターンを認識
        if let Some(captures) = Regex::new(r"^(\w+)\s*\(\s*(.*)\s*\)$").unwrap().captures(expr) {
            // 関数呼び出し: func(args)
            let func_name = captures.get(1).unwrap().as_str();
            let args_str = captures.get(2).unwrap().as_str();
            
            let args = args_str.split(',')
                .map(|arg| arg.trim())
                .filter(|arg| !arg.is_empty())
                .map(|arg| self.parse_expression(arg))
                .collect::<Result<Vec<_>, _>>()?;
            
            return Ok(ExpressionAST::FunctionCall {
                function: Box::new(ExpressionAST::Variable(func_name.to_string())),
                arguments: args,
            });
        } else if let Some(captures) = Regex::new(r"^fn\s*\((.*)\)\s*->\s*(.+)$").unwrap().captures(expr) {
            // 関数定義: fn(params) -> return_type
            let params_str = captures.get(1).unwrap().as_str();
            let return_type_str = captures.get(2).unwrap().as_str();
            
            let params = params_str.split(',')
                .map(|param| param.trim())
                .filter(|param| !param.is_empty())
                .map(|param| {
                    if let Some((name, type_str)) = param.split_once(':') {
                        Ok((name.trim().to_string(), self.parse_type(type_str.trim())?))
                    } else {
                        Err(ErrorDiagnostic::new(
                            format!("Invalid parameter format: {}", param),
                            "Parser".to_string(),
                            None,
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            
            let return_type = self.parse_type(return_type_str)?;
            
            return Ok(ExpressionAST::Lambda {
                parameters: params,
                return_type: Box::new(return_type),
                body: Box::new(ExpressionAST::Unit), // 実際の実装ではボディも解析
            });
        } else if expr.contains("->") {
            // 関数型: T1 -> T2
            let parts: Vec<&str> = expr.split("->").map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(ErrorDiagnostic::new(
                    format!("Invalid function type syntax: {}", expr),
                    "Parser".to_string(),
                    None,
                ));
            }
            
            let param_type = self.parse_type(parts[0])?;
            let return_type = self.parse_type(parts[1])?;
            
            return Ok(ExpressionAST::TypeAnnotation {
                expression: Box::new(ExpressionAST::Unit), // プレースホルダー
                type_: Type::Function(vec![param_type], Box::new(return_type)),
            });
        } else if let Some(captures) = Regex::new(r"^let\s+(\w+)\s*=\s*(.+)$").unwrap().captures(expr) {
            // 変数定義: let x = expr
            let var_name = captures.get(1).unwrap().as_str();
            let value_expr = captures.get(2).unwrap().as_str();
            
            let value_ast = self.parse_expression(value_expr)?;
            
            return Ok(ExpressionAST::Let {
                name: var_name.to_string(),
                value: Box::new(value_ast),
                body: Box::new(ExpressionAST::Unit), // 実際の実装ではボディも解析
            });
        } else if expr.chars().all(|c| c.is_numeric() || c == '.') {
            // 数値リテラル
            if expr.contains('.') {
                return Ok(ExpressionAST::Literal(LiteralValue::Float(
                    expr.parse().map_err(|_| ErrorDiagnostic::new(
                        format!("Invalid float literal: {}", expr),
                        "Parser".to_string(),
                        None,
                    ))?
                )));
            } else {
                return Ok(ExpressionAST::Literal(LiteralValue::Integer(
                    expr.parse().map_err(|_| ErrorDiagnostic::new(
                        format!("Invalid integer literal: {}", expr),
                        "Parser".to_string(),
                        None,
                    ))?
                )));
            }
        } else if expr.starts_with("\"") && expr.ends_with("\"") {
            // 文字列リテラル
            return Ok(ExpressionAST::Literal(LiteralValue::String(
                expr[1..expr.len()-1].to_string()
            )));
        } else if expr == "true" || expr == "false" {
            // 真偽値リテラル
            return Ok(ExpressionAST::Literal(LiteralValue::Boolean(
                expr == "true"
            )));
        } else if self.type_env.contains_key(expr) {
            // 変数参照
            return Ok(ExpressionAST::Variable(expr.to_string()));
        }
        
        // 解析できない場合
        Err(ErrorDiagnostic::new(
            format!("Failed to parse expression: {}", expr),
            "Parser".to_string(),
            None,
        ))
    }
    
    /// 型文字列を解析して型を生成
    fn parse_type(&self, type_str: &str) -> Result<Type, ErrorDiagnostic> {
        // 基本型
        match type_str.trim() {
            "int" | "Int" => return Ok(Type::Named("Int".to_string())),
            "float" | "Float" => return Ok(Type::Named("Float".to_string())),
            "string" | "String" => return Ok(Type::Named("String".to_string())),
            "bool" | "Bool" => return Ok(Type::Named("Bool".to_string())),
            "unit" | "Unit" | "()" => return Ok(Type::Named("Unit".to_string())),
            _ => {}
        }
        
        // 関数型
        if type_str.contains("->") {
            let parts: Vec<&str> = type_str.split("->").map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(ErrorDiagnostic::new(
                    format!("Invalid function type syntax: {}", type_str),
                    "TypeParser".to_string(),
                    None,
                ));
            }
            
            // パラメータ型の解析
            let param_type = self.parse_type(parts[0])?;
            
            // 戻り値型の解析
            let return_type = self.parse_type(parts[1])?;
            
            return Ok(Type::Function(vec![param_type], Box::new(return_type)));
        }
        
        // ジェネリック型
        if let Some(captures) = Regex::new(r"^(\w+)<(.+)>$").unwrap().captures(type_str) {
            let base_type = captures.get(1).unwrap().as_str();
            let type_args_str = captures.get(2).unwrap().as_str();
            
            let type_args = type_args_str.split(',')
                .map(|arg| arg.trim())
                .filter(|arg| !arg.is_empty())
                .map(|arg| self.parse_type(arg))
                .collect::<Result<Vec<_>, _>>()?;
            
            return Ok(Type::Generic(base_type.to_string(), type_args));
        }
        
        // 型変数または名前付き型
        if type_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
            if type_str.starts_with(|c: char| c.is_uppercase()) {
                return Ok(Type::Named(type_str.to_string()));
            } else {
                return Ok(Type::TypeParam(type_str.to_string()));
            }
        }
        
        // タプル型
        if type_str.starts_with('(') && type_str.ends_with(')') {
            let inner = &type_str[1..type_str.len()-1];
            if inner.is_empty() {
                return Ok(Type::Named("Unit".to_string()));
            }
            
            let mut types = Vec::new();
            let mut current = String::new();
            let mut depth = 0;
            
            for c in inner.chars() {
                match c {
                    '(' | '<' | '[' => {
                        depth += 1;
                        current.push(c);
                    },
                    ')' | '>' | ']' => {
                        depth -= 1;
                        current.push(c);
                    },
                    ',' if depth == 0 => {
                        if !current.is_empty() {
                            types.push(self.parse_type(&current)?);
                            current.clear();
                        }
                    },
                    _ => current.push(c),
                }
            }
            
            if !current.is_empty() {
                types.push(self.parse_type(&current)?);
            }
            
            if types.len() == 1 {
                return Ok(types.remove(0));
            } else {
                return Ok(Type::Tuple(types));
            }
        }
        
        // 解析できない場合
        Err(ErrorDiagnostic::new(
            format!("Failed to parse type: {}", type_str),
            "TypeParser".to_string(),
            None,
        ))
    }
    
    /// ASTに基づいて型推論を実行
    fn infer_ast(&mut self, ast: &ExpressionAST) -> Result<Type, Vec<ErrorDiagnostic>> {
        match ast {
            ExpressionAST::Variable(name) => {
                if let Some(type_) = self.type_env.get(name) {
                    // 環境から型を取得
                    Ok(type_.clone())
                } else {
                    // 未定義の変数
                    Err(vec![ErrorDiagnostic::new(
                        format!("Undefined variable: {}", name),
                        "TypeInference".to_string(),
                        None,
                    )])
                }
            },
            ExpressionAST::Literal(value) => {
                // リテラル値の型を決定
                match value {
                    LiteralValue::Integer(_) => Ok(Type::Named("Int".to_string())),
                    LiteralValue::Float(_) => Ok(Type::Named("Float".to_string())),
                    LiteralValue::String(_) => Ok(Type::Named("String".to_string())),
                    LiteralValue::Boolean(_) => Ok(Type::Named("Bool".to_string())),
                    LiteralValue::Unit => Ok(Type::Named("Unit".to_string())),
                }
            },
            ExpressionAST::FunctionCall { function, arguments } => {
                // 関数の型を推論
                let func_type = self.infer_ast(function)?;
                
                // 引数の型を推論
                let mut arg_types = Vec::new();
                for arg in arguments {
                    arg_types.push(self.infer_ast(arg)?);
                }
                
                // 関数適用の型チェック
                match func_type {
                    Type::Function(param_types, return_type) => {
                        // 引数の数をチェック
                        if param_types.len() != arg_types.len() {
                            return Err(vec![ErrorDiagnostic::new(
                                format!("Function expected {} arguments but got {}", 
                                        param_types.len(), arg_types.len()),
                                "TypeInference".to_string(),
                                None,
                            )]);
                        }
                        
                        // 各引数の型を単一化
                        for (param_type, arg_type) in param_types.iter().zip(arg_types.iter()) {
                            self.unify(param_type, arg_type)?;
                        }
                        
                        // 戻り値の型を返す
                        Ok(*return_type)
                    },
                    _ => {
                        Err(vec![ErrorDiagnostic::new(
                            format!("Cannot apply non-function type: {}", func_type.to_string()),
                            "TypeInference".to_string(),
                            None,
                        )])
                    }
                }
            },
            ExpressionAST::Lambda { parameters, return_type, body } => {
                // 新しいスコープを作成
                let mut new_env = self.type_env.clone();
                
                // パラメータを環境に追加
                let mut param_types = Vec::new();
                for (name, type_) in parameters {
                    param_types.push(type_.clone());
                    new_env.insert(name.clone(), type_.clone());
                }
                
                // 元の環境を保存
                let old_env = std::mem::replace(&mut self.type_env, new_env);
                
                // ボディの型を推論
                let body_type = self.infer_ast(body)?;
                
                // 環境を復元
                self.type_env = old_env;
                
                // 戻り値型と推論された型を単一化
                self.unify(&body_type, return_type)?;
                
                // 関数型を返す
                Ok(Type::Function(param_types, Box::new(body_type)))
            },
            ExpressionAST::Let { name, value, body } => {
                // 値の型を推論
                let value_type = self.infer_ast(value)?;
                
                // 環境に変数を追加
                let old_env = self.type_env.clone();
                self.type_env.insert(name.clone(), value_type);
                
                // ボディの型を推論
                let body_type = self.infer_ast(body)?;
                
                // 環境を復元（実際の実装ではスコープ管理が必要）
                self.type_env = old_env;
                
                Ok(body_type)
            },
            ExpressionAST::TypeAnnotation { expression, type_ } => {
                // 式の型を推論
                let inferred_type = self.infer_ast(expression)?;
                
                // アノテーションと推論された型を単一化
                self.unify(&inferred_type, type_)?;
                
                Ok(type_.clone())
            },
            ExpressionAST::Unit => {
                Ok(Type::Named("Unit".to_string()))
            },
            // 他のAST要素に対する型推論...
        }
    }
    
    /// 制約を解決
    fn solve_constraints(&mut self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // 全ての制約を処理
        for constraint in self.constraints.clone() {
            match constraint {
                TypeConstraintImpl::Equality(t1, t2) => {
                    if let Err(mut errs) = self.unify(&t1, &t2) {
                        errors.append(&mut errs);
                    }
                },
                TypeConstraintImpl::Subtype(sub, sup) => {
                    if let Err(mut errs) = self.check_subtype(&sub, &sup) {
                        errors.append(&mut errs);
                    }
                },
                // 他の制約タイプ...
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// サブタイプ関係をチェック
    fn check_subtype(&self, sub: &Type, sup: &Type) -> Result<(), Vec<ErrorDiagnostic>> {
        // 同じ型は常にサブタイプ関係
        if sub == sup {
            return Ok(());
        }
        
        // 登録されたサブタイプ関係をチェック
        for relation in &self.subtype_relations {
            if self.types_match(&relation.subtype, sub) && self.types_match(&relation.supertype, sup) {
                return Ok(());
            }
        }
        
        // 関数型のサブタイプ関係（共変・反変）
        if let (Type::Function(params1, ret1), Type::Function(params2, ret2)) = (sub, sup) {
            if params1.len() != params2.len() {
                return Err(vec![ErrorDiagnostic::new(
                    format!("Function types have different parameter counts: {} and {}", 
                            params1.len(), params2.len()),
                    "SubtypeCheck".to_string(),
                    None,
                )]);
            }
            
            // パラメータは反変（supertype -> subtype）
            for (p1, p2) in params2.iter().zip(params1.iter()) {
                self.check_subtype(p1, p2)?;
            }
            
            // 戻り値型は共変（subtype -> supertype）
            self.check_subtype(ret1, ret2)?;
            
            return Ok(());
        }
        
        // サブタイプ関係が見つからない
        Err(vec![ErrorDiagnostic::new(
            format!("{} is not a subtype of {}", sub.to_string(), sup.to_string()),
            "SubtypeCheck".to_string(),
            None,
        )])
    }
    
    /// 型パターンマッチング
    fn types_match(&self, pattern: &Type, concrete: &Type) -> bool {
        match (pattern, concrete) {
            (Type::TypeParam(_), _) => true, // 型変数は任意の型にマッチ
            (Type::Named(n1), Type::Named(n2)) => n1 == n2,
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return false;
                }
                
                params1.iter().zip(params2.iter()).all(|(p1, p2)| self.types_match(p1, p2)) &&
                self.types_match(ret1, ret2)
            },
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                
                types1.iter().zip(types2.iter()).all(|(t1, t2)| self.types_match(t1, t2))
            },
            (Type::Generic(name1, args1), Type::Generic(name2, args2)) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return false;
                }
                
                args1.iter().zip(args2.iter()).all(|(a1, a2)| self.types_match(a1, a2))
            },
            _ => false,
        }
    }
    
    /// 型を正規化（型変数を解決）
    fn normalize_type(&self, type_: &Type) -> Result<Type, Vec<ErrorDiagnostic>> {
        match type_ {
            Type::TypeParam(name) => {
                // 型環境から解決された型を探す
                if let Some(resolved) = self.type_env.get(name) {
                    // 循環参照を防ぐ
                    if let Type::TypeParam(resolved_name) = resolved {
                        if resolved_name == name {
                            return Ok(type_.clone());
                        }
                    }
                    
                    // 再帰的に正規化
                    self.normalize_type(resolved)
                } else {
                    // 解決されていない型変数はそのまま
                    Ok(type_.clone())
                }
            },
            Type::Function(params, ret) => {
                // パラメータ型を正規化
                let mut normalized_params = Vec::new();
                for param in params {
                    normalized_params.push(self.normalize_type(param)?);
                }
                
                // 戻り値型を正規化
                let normalized_ret = self.normalize_type(ret)?;
                
                Ok(Type::Function(normalized_params, Box::new(normalized_ret)))
            },
            Type::Tuple(types) => {
                // タプル要素を正規化
                let mut normalized_types = Vec::new();
                for t in types {
                    normalized_types.push(self.normalize_type(t)?);
                }
                
                Ok(Type::Tuple(normalized_types))
            },
            Type::Generic(name, args) => {
                // 型引数を正規化
                let mut normalized_args = Vec::new();
                for arg in args {
                    normalized_args.push(self.normalize_type(arg)?);
                }
                
                Ok(Type::Generic(name.clone(), normalized_args))
            },
            // 他の型はそのまま
            _ => Ok(type_.clone()),
        }
    }
    
    /// 式がルールにマッチするかチェック
    fn matches_rule(&self, expr: &str, rule: &TypeRuleImpl) -> bool {
        // 実際の実装ではより複雑なマッチングロジックが必要
        expr == rule.conclusion.expression
    }
    
    /// 型単一化（型変数を解決）
    fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), Vec<ErrorDiagnostic>> {
        // ヒンドリー・ミルナー型推論の単一化アルゴリズム
        match (t1, t2) {
            (Type::TypeParam(v1), Type::TypeParam(v2)) if v1 == v2 => {
                // 同じ型変数同士は単一化可能
                Ok(())
            },
            (Type::TypeParam(v), t) => {
                // 型変数と他の型の単一化
                // 出現チェック（occurs check）- 循環参照を防ぐ
                if self.occurs_in(v, t) {
                    return Err(vec![ErrorDiagnostic::new(
                        format!("Recursive type detected: {} occurs in {}", v, t.to_string()),
                        "TypeUnification".to_string(),
                        None,
                    )]);
                }
                
                // 型環境を更新
                self.type_env.insert(v.clone(), t.clone());
                Ok(())
            },
            (t, Type::TypeParam(v)) => {
                // 対称性のため、順序を入れ替えて再帰呼び出し
                self.unify(t2, t1)
            },
            (Type::Named(n1), Type::Named(n2)) if n1 == n2 => {
                // 同じ名前の型は単一化可能
                Ok(())
            },
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                // 関数型の単一化
                if params1.len() != params2.len() {
                    return Err(vec![ErrorDiagnostic::new(
                        format!("Cannot unify function types with different parameter counts: {} and {}", 
                                params1.len(), params2.len()),
                        "TypeUnification".to_string(),
                        None,
                    )]);
                }
                
                // パラメータの単一化
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2)?;
                }
                
                // 戻り値型の単一化
                self.unify(ret1, ret2)
            },
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                // タプル型の単一化
                if types1.len() != types2.len() {
                    return Err(vec![ErrorDiagnostic::new(
                        format!("Cannot unify tuple types with different lengths: {} and {}", 
                                types1.len(), types2.len()),
                        "TypeUnification".to_string(),
                        None,
                    )]);
                }
                
                // 各要素の単一化
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify(t1, t2)?;
                }
                
                Ok(())
            },
            (Type::Generic(name1, args1), Type::Generic(name2, args2)) => {
                // ジェネリック型の単一化
                if name1 != name2 {
                    return Err(vec![ErrorDiagnostic::new(
                        format!("Cannot unify different generic types: {} and {}", 
                                name1, name2),
                        "TypeUnification".to_string(),
                        None,
                    )]);
                }
                
                if args1.len() != args2.len() {
                    return Err(vec![ErrorDiagnostic::new(
                        format!("Cannot unify generic types with different argument counts: {} and {}", 
                                args1.len(), args2.len()),
                        "TypeUnification".to_string(),
                        None,
                    )]);
                }
                
                // 型引数の単一化
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                
                Ok(())
            },
            // 他の型に対する単一化ルール...
            _ => {
                // 単一化できない型の組み合わせ
                Err(vec![ErrorDiagnostic::new(
                    format!("Cannot unify types {} and {}", t1.to_string(), t2.to_string()),
                    "TypeUnification".to_string(),
                    None,
                )])
            }
        }
    }
    
    /// 型変数が型に出現するかチェック（occurs check）
    fn occurs_in(&self, var: &str, type_: &Type) -> bool {
        match type_ {
            Type::TypeParam(v) => var == v,
            Type::Function(params, ret) => {
                params.iter().any(|p| self.occurs_in(var, p)) || self.occurs_in(var, ret)
            },
            Type::Tuple(types) => {
                types.iter().any(|t| self.occurs_in(var, t))
            },
            Type::Generic(_, args) => {
                args.iter().any(|a| self.occurs_in(var, a))
            },
            _ => false,
        }
    }
}

/// 式のAST
#[derive(Debug, Clone)]
enum ExpressionAST {
    Variable(String),
    Literal(LiteralValue),
    FunctionCall {
        function: Box<ExpressionAST>,
        arguments: Vec<ExpressionAST>,
    },
    Lambda {
        parameters: Vec<(String, Type)>,
        return_type: Box<Type>,
        body: Box<ExpressionAST>,
    },
    Let {
        name: String,
        value: Box<ExpressionAST>,
        body: Box<ExpressionAST>,
    },
    TypeAnnotation {
        expression: Box<ExpressionAST>,
        type_: Type,
    },
    Unit,
}

/// リテラル値
#[derive(Debug, Clone)]
enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
}

/// サブタイプ関係の実装
#[derive(Debug, Clone)]
pub struct SubtypeRelationImpl {
    /// サブタイプ
    pub subtype: Type,
    
    /// スーパータイプ
    pub supertype: Type,
    
    /// 関係の名前
    pub name: Option<String>,
}

impl SubtypeRelationImpl {
    /// サブタイプ関係を作成
    pub fn new(subtype: Type, supertype: Type) -> Self {
        SubtypeRelationImpl {
            subtype,
            supertype,
            name: None,
        }
    }
    
    /// 名前付きサブタイプ関係を作成
    pub fn with_name(subtype: Type, supertype: Type, name: &str) -> Self {
        SubtypeRelationImpl {
            subtype,
            supertype,
            name: Some(name.to_string()),
        }
    }
    
    /// サブタイプ関係を文字列に変換
    pub fn to_string(&self) -> String {
        if let Some(name) = &self.name {
            format!("{}: {} <: {} ({})", name, self.subtype.to_string(), self.supertype.to_string(), name)
        } else {
            format!("{} <: {}", self.subtype.to_string(), self.supertype.to_string())
        }
    }
}

/// 型システムDSLのマクロ
#[macro_export]
macro_rules! type_system {
    // 型システム定義全体
    ($name:ident {
        $($content:tt)*
    }) => {
        {
            let mut type_system = TypeSystem::new();
            $($crate::type_system_item!(type_system, $content);)*
            type_system
        }
    };
}

/// 型システムアイテムのマクロヘルパー
#[macro_export]
macro_rules! type_system_item {
    // 基本型定義
    ($ts:ident, base_type $name:ident = $type:ident $( ( $($args:expr),* ) )?;) => {
        {
            let base_type = BaseType::$type $( ( $($args),* ) )?;
            $ts.add_base_type(stringify!($name), base_type);
        }
    };
    
    // 型エイリアス
    ($ts:ident, type $name:ident = $type:expr;) => {
        {
            let type_def = TypeDefinition {
                name: stringify!($name).to_string(),
                type_parameters: Vec::new(),
                structure: $type,
                constraints: Vec::new(),
            };
            $ts.composite_types.push(type_def);
        }
    };
    
    // 型チェックルール
    ($ts:ident, rule $name:ident: $expr:expr : $type:expr;) => {
        {
            let rule = TypeRuleImpl {
                name: stringify!($name).to_string(),
                premises: Vec::new(),
                conclusion: TypeJudgmentImpl {
                    expression: $expr.to_string(),
                    type_: $type,
                },
            };
            $ts.typing_rules.push(TypeRule::from(rule));
        }
    };
    
    // 前提条件付き型チェックルール
    ($ts:ident, rule $name:ident: $($premise:expr : $premise_type:expr),+ => $expr:expr : $type:expr;) => {
        {
            let mut premises = Vec::new();
            $(
                premises.push(TypeJudgmentImpl {
                    expression: $premise.to_string(),
                    type_: $premise_type,
                });
            )+
            
            let rule = TypeRuleImpl {
                name: stringify!($name).to_string(),
                premises,
                conclusion: TypeJudgmentImpl {
                    expression: $expr.to_string(),
                    type_: $type,
                },
            };
            $ts.typing_rules.push(TypeRule::from(rule));
        }
    };
    
    // サブタイプ関係
    ($ts:ident, subtype $sub:expr <: $super:expr;) => {
        {
            let relation = SubtypeRelationImpl::new($sub, $super);
            $ts.subtyping_relation.push(SubtypeRelation::from(relation));
        }
    };
    
    // 名前付きサブタイプ関係
    ($ts:ident, subtype $name:ident: $sub:expr <: $super:expr;) => {
        {
            let relation = SubtypeRelationImpl::with_name($sub, $super, stringify!($name));
            $ts.subtyping_relation.push(SubtypeRelation::from(relation));
        }
    };
    
    // 型制約
    ($ts:ident, constraint $left:expr = $right:expr;) => {
        {
            let constraint = TypeConstraintImpl::equal($left, $right);
            $ts.constraints.push(TypeConstraint::from(constraint));
        }
    };
    
    // サブタイプ制約
    ($ts:ident, constraint $left:expr <: $right:expr;) => {
        {
            let constraint = TypeConstraintImpl::subtype($left, $right);
            $ts.constraints.push(TypeConstraint::from(constraint));
        }
    };
    
    // 型クラス制約
    ($ts:ident, constraint $type:expr : $class:ident;) => {
        {
            let constraint = TypeConstraintImpl::type_class($type, stringify!($class));
            $ts.constraints.push(TypeConstraint::from(constraint));
        }
    };
}

/// 型定義DSLのヘルパー関数
pub mod dsl {
    use super::*;
    
    /// 整数型を作成
    pub fn integer(bit_width: usize, signed: bool) -> BaseType {
        BaseType::Integer { bit_width, signed }
    }
    
    /// 浮動小数点型を作成
    pub fn float(bit_width: usize) -> BaseType {
        BaseType::Float { bit_width }
    }
    
    /// 文字型を作成
    pub fn char(unicode: bool) -> BaseType {
        BaseType::Char { unicode }
    }
    
    /// 真偽値型を作成
    pub fn boolean() -> BaseType {
        BaseType::Boolean
    }
    
    /// 単位型を作成
    pub fn unit() -> BaseType {
        BaseType::Unit
    }
    
    /// ボトム型を作成
    pub fn bottom() -> BaseType {
        BaseType::Bottom
    }
    
    /// トップ型を作成
    pub fn top() -> BaseType {
        BaseType::Top
    }
    
    /// アドレス型を作成
    pub fn address(address_space: usize) -> BaseType {
        BaseType::Address { address_space }
    }
    
    /// 列挙型を作成
    pub fn enum_type(values: Vec<&str>) -> BaseType {
        BaseType::Enum { 
            values: values.iter().map(|v| v.to_string()).collect() 
        }
    }
} 