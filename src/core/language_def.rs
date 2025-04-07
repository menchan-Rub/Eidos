// 言語定義のコア構造体とトレイト
// Eidosの「世界一の汎用言語を作るためのDSL」の基盤

use std::collections::{HashMap, HashSet};
use crate::types::{Type, TypeConstraint, TypeRule, BaseType};
use crate::syntax::{SyntaxRule, SyntaxDefinition, ProductionRule};
use crate::semantics::{SemanticsRule, ExecutionModel};
use crate::memory::{MemoryModel, OwnershipModel, AllocationStrategy};
use crate::optimization::{OptimizationPass, OptimizationLevel};
use crate::error::ErrorDiagnostic;

/// 言語定義のルートとなる構造体
#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    /// 言語の名前
    pub name: String,
    
    /// 言語のバージョン
    pub version: String,
    
    /// 言語の構文定義
    pub syntax: SyntaxDefinition,
    
    /// 言語の型システム
    pub type_system: TypeSystem,
    
    /// 言語のセマンティクス
    pub semantics: SemanticsDefinition,
    
    /// 言語のメモリ管理モデル
    pub memory_model: MemoryModel,
    
    /// 言語の最適化パス
    pub optimizations: Vec<OptimizationPass>,
    
    /// 言語のエラー報告システム
    pub error_system: ErrorSystem,
    
    /// 言語の標準ライブラリ定義
    pub std_library: StdLibraryDefinition,
    
    /// 言語の拡張ポイント（プラグイン等）
    pub extension_points: HashMap<String, ExtensionPoint>,
}

impl LanguageDefinition {
    /// 新しい言語定義を作成
    pub fn new(name: &str) -> Self {
        LanguageDefinition {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            syntax: SyntaxDefinition::new(),
            type_system: TypeSystem::new(),
            semantics: SemanticsDefinition::new(),
            memory_model: MemoryModel::default(),
            optimizations: Vec::new(),
            error_system: ErrorSystem::new(),
            std_library: StdLibraryDefinition::new(),
            extension_points: HashMap::new(),
        }
    }
    
    /// 言語定義を検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // 構文定義の検証
        if let Err(syntax_errors) = self.syntax.validate() {
            errors.extend(syntax_errors);
        }
        
        // 型システムの検証
        if let Err(type_errors) = self.type_system.validate() {
            errors.extend(type_errors);
        }
        
        // セマンティクスの検証
        if let Err(semantics_errors) = self.semantics.validate() {
            errors.extend(semantics_errors);
        }
        
        // メモリモデルの検証
        if let Err(memory_errors) = self.memory_model.validate() {
            errors.extend(memory_errors);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 言語定義からコンパイラを生成
    pub fn generate_compiler(&self) -> Result<CompilerDefinition, Vec<ErrorDiagnostic>> {
        // 言語定義を検証
        self.validate()?;
        
        // コンパイラの各フェーズを構築
        let lexer = self.syntax.generate_lexer();
        let parser = self.syntax.generate_parser();
        let type_checker = self.type_system.generate_type_checker();
        let code_generator = CodeGenerator::new(self);
        
        Ok(CompilerDefinition {
            name: format!("{}-compiler", self.name),
            version: self.version.clone(),
            lexer,
            parser,
            type_checker,
            optimizer: Optimizer::from_passes(self.optimizations.clone()),
            code_generator,
        })
    }
}

/// 型システム定義
#[derive(Debug, Clone)]
pub struct TypeSystem {
    /// 基本型の定義
    pub base_types: HashMap<String, BaseType>,
    
    /// 複合型の定義ルール
    pub composite_types: Vec<TypeDefinition>,
    
    /// 型チェックのルール
    pub typing_rules: Vec<TypeRule>,
    
    /// 型変換のルール
    pub conversion_rules: Vec<ConversionRule>,
    
    /// 型推論のルール
    pub inference_rules: Vec<InferenceRule>,
    
    /// サブタイプ関係
    pub subtyping_relation: Vec<SubtypeRelation>,
    
    /// 型制約
    pub constraints: Vec<TypeConstraint>,
}

impl TypeSystem {
    /// 新しい型システムを作成
    pub fn new() -> Self {
        TypeSystem {
            base_types: HashMap::new(),
            composite_types: Vec::new(),
            typing_rules: Vec::new(),
            conversion_rules: Vec::new(),
            inference_rules: Vec::new(),
            subtyping_relation: Vec::new(),
            constraints: Vec::new(),
        }
    }
    
    /// 基本型を追加
    pub fn add_base_type(&mut self, name: &str, base_type: BaseType) {
        self.base_types.insert(name.to_string(), base_type);
    }
    
    /// 型チェックルールを追加
    pub fn add_typing_rule(&mut self, rule: TypeRule) {
        self.typing_rules.push(rule);
    }
    
    /// 型システムを検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // 基本型の検証
        if self.base_types.is_empty() {
            errors.push(ErrorDiagnostic::new(
                "Type system must define at least one base type".to_string(),
                "TypeSystem".to_string(),
                None,
            ));
        }
        
        // 型チェックルールの検証
        for rule in &self.typing_rules {
            if let Err(rule_errors) = rule.validate(&self.base_types) {
                errors.extend(rule_errors);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 型システムから型チェッカーを生成
    pub fn generate_type_checker(&self) -> TypeChecker {
        TypeChecker::from_type_system(self)
    }
}

/// セマンティクス定義
#[derive(Debug, Clone)]
pub struct SemanticsDefinition {
    /// 実行モデル
    pub execution_model: ExecutionModel,
    
    /// セマンティクスルール
    pub rules: Vec<SemanticsRule>,
    
    /// 評価戦略
    pub evaluation_strategy: EvaluationStrategy,
    
    /// 制御フロー構造
    pub control_flow: ControlFlowDefinition,
    
    /// 並行処理モデル（オプション）
    pub concurrency_model: Option<ConcurrencyModel>,
    
    /// エフェクト系（オプション）
    pub effects_system: Option<EffectsSystem>,
}

impl SemanticsDefinition {
    /// 新しいセマンティクス定義を作成
    pub fn new() -> Self {
        SemanticsDefinition {
            execution_model: ExecutionModel::Imperative,
            rules: Vec::new(),
            evaluation_strategy: EvaluationStrategy::StrictEvaluation,
            control_flow: ControlFlowDefinition::new(),
            concurrency_model: None,
            effects_system: None,
        }
    }
    
    /// セマンティクスを検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // セマンティクスルールの検証
        for rule in &self.rules {
            if let Err(rule_errors) = rule.validate() {
                errors.extend(rule_errors);
            }
        }
        
        // 制御フローの検証
        if let Err(cf_errors) = self.control_flow.validate() {
            errors.extend(cf_errors);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// エラー報告システム
#[derive(Debug, Clone)]
pub struct ErrorSystem {
    /// エラーメッセージのテンプレート
    pub message_templates: HashMap<String, String>,
    
    /// エラーコード体系
    pub error_codes: HashMap<String, ErrorCode>,
    
    /// ヒントとフィックスの提案システム
    pub suggestions: bool,
}

impl ErrorSystem {
    /// 新しいエラー報告システムを作成
    pub fn new() -> Self {
        ErrorSystem {
            message_templates: HashMap::new(),
            error_codes: HashMap::new(),
            suggestions: true,
        }
    }
}

/// 標準ライブラリ定義
#[derive(Debug, Clone)]
pub struct StdLibraryDefinition {
    /// コアモジュール
    pub core_modules: Vec<ModuleDefinition>,
    
    /// 標準型
    pub standard_types: HashSet<String>,
    
    /// 標準関数
    pub standard_functions: HashMap<String, FunctionDefinition>,
    
    /// プリミティブ演算
    pub primitives: HashSet<String>,
}

impl StdLibraryDefinition {
    /// 新しい標準ライブラリ定義を作成
    pub fn new() -> Self {
        StdLibraryDefinition {
            core_modules: Vec::new(),
            standard_types: HashSet::new(),
            standard_functions: HashMap::new(),
            primitives: HashSet::new(),
        }
    }
}

/// 拡張ポイント
#[derive(Debug, Clone)]
pub struct ExtensionPoint {
    /// 拡張ポイントの名前
    pub name: String,
    
    /// 拡張ポイントの説明
    pub description: String,
    
    /// 拡張ポイントのインターフェース
    pub interface_trait: String,
    
    /// 拡張ポイントの優先度
    pub priority: ExtensionPriority,
}

/// 拡張ポイントの優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExtensionPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Highest = 4,
}

/// コンパイラ定義
#[derive(Debug, Clone)]
pub struct CompilerDefinition {
    /// コンパイラの名前
    pub name: String,
    
    /// コンパイラのバージョン
    pub version: String,
    
    /// 字句解析器
    pub lexer: Lexer,
    
    /// 構文解析器
    pub parser: Parser,
    
    /// 型チェッカー
    pub type_checker: TypeChecker,
    
    /// 最適化器
    pub optimizer: Optimizer,
    
    /// コード生成器
    pub code_generator: CodeGenerator,
}

/// 字句解析器
#[derive(Debug, Clone)]
pub struct Lexer {
    /// トークン定義
    pub token_definitions: Vec<TokenDefinition>,
    
    /// スキップするトークン（コメント、空白など）
    pub skip_tokens: HashSet<String>,
}

/// トークン定義
#[derive(Debug, Clone)]
pub struct TokenDefinition {
    /// トークンの名前
    pub name: String,
    
    /// トークンの正規表現パターン
    pub pattern: String,
    
    /// トークンの優先度
    pub priority: u32,
}

/// 構文解析器
#[derive(Debug, Clone)]
pub struct Parser {
    /// 文法ルール
    pub grammar_rules: Vec<ProductionRule>,
    
    /// パーサーの種類
    pub parser_type: ParserType,
    
    /// エラー回復戦略
    pub error_recovery: ErrorRecoveryStrategy,
}

/// パーサーの種類
#[derive(Debug, Clone)]
pub enum ParserType {
    /// 再帰下降パーサー
    RecursiveDescent,
    
    /// 予測的パーサー (LL)
    PredictiveLL(usize),
    
    /// LR系パーサー
    LR(LRVariant),
}

/// LRパーサーのバリアント
#[derive(Debug, Clone)]
pub enum LRVariant {
    /// シンプルLR
    SLR,
    
    /// 先読みLR
    LALR,
    
    /// 正準LR
    CanonicalLR,
}

/// エラー回復戦略
#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    /// パニックモード
    Panic,
    
    /// フレーズレベルの回復
    PhraseLevel,
    
    /// エラー生成
    ErrorProduction,
}

/// 型チェッカー
#[derive(Debug, Clone)]
pub struct TypeChecker {
    /// 型チェックのルール
    pub typing_rules: Vec<TypeRule>,
    
    /// 型推論の使用有無
    pub use_inference: bool,
    
    /// 型エラーメッセージ
    pub error_messages: HashMap<String, String>,
}

impl TypeChecker {
    /// 型システムから型チェッカーを生成
    pub fn from_type_system(type_system: &TypeSystem) -> Self {
        TypeChecker {
            typing_rules: type_system.typing_rules.clone(),
            use_inference: !type_system.inference_rules.is_empty(),
            error_messages: HashMap::new(),
        }
    }
}

/// 最適化器
#[derive(Debug, Clone)]
pub struct Optimizer {
    /// 最適化パス
    pub passes: Vec<OptimizationPass>,
    
    /// 最適化レベル
    pub level: OptimizationLevel,
    
    /// ターゲット固有の最適化
    pub target_specific: HashMap<String, Vec<OptimizationPass>>,
}

impl Optimizer {
    /// 最適化パスから最適化器を生成
    pub fn from_passes(passes: Vec<OptimizationPass>) -> Self {
        Optimizer {
            passes,
            level: OptimizationLevel::Default,
            target_specific: HashMap::new(),
        }
    }
}

/// コード生成器
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    /// サポートするターゲット
    pub supported_targets: HashSet<String>,
    
    /// ターゲットごとのバックエンド
    pub backends: HashMap<String, CodeGenBackend>,
    
    /// ABI定義
    pub abi_definitions: HashMap<String, ABIDefinition>,
}

impl CodeGenerator {
    /// 言語定義からコード生成器を生成
    pub fn new(lang_def: &LanguageDefinition) -> Self {
        let mut supported_targets = HashSet::new();
        supported_targets.insert("llvm".to_string());
        supported_targets.insert("wasm".to_string());
        supported_targets.insert("native".to_string());
        supported_targets.insert("js".to_string());
        supported_targets.insert("bytecode".to_string());
        
        // バックエンドの初期化
        let mut backends = HashMap::new();
        
        // LLVMバックエンドの設定
        let llvm_backend = CodeGenBackend {
            name: "llvm".to_string(),
            architectures: ["x86_64", "aarch64", "arm", "riscv64", "wasm32"]
                .iter().map(|s| s.to_string()).collect(),
            instruction_selection: InstructionSelectionStrategy::DAGBased,
            register_allocation: RegisterAllocationStrategy::GraphColoring,
        };
        backends.insert("llvm".to_string(), llvm_backend);
        
        // WebAssemblyバックエンドの設定
        let wasm_backend = CodeGenBackend {
            name: "wasm".to_string(),
            architectures: ["wasm32", "wasm64"].iter().map(|s| s.to_string()).collect(),
            instruction_selection: InstructionSelectionStrategy::TreeRewriting,
            register_allocation: RegisterAllocationStrategy::LinearScan,
        };
        backends.insert("wasm".to_string(), wasm_backend);
        
        // ネイティブバックエンドの設定
        let native_backend = CodeGenBackend {
            name: "native".to_string(),
            architectures: ["x86_64", "aarch64"].iter().map(|s| s.to_string()).collect(),
            instruction_selection: InstructionSelectionStrategy::PatternMatching,
            register_allocation: RegisterAllocationStrategy::GraphColoring,
        };
        backends.insert("native".to_string(), native_backend);
        
        // JavaScriptバックエンドの設定
        let js_backend = CodeGenBackend {
            name: "js".to_string(),
            architectures: ["js"].iter().map(|s| s.to_string()).collect(),
            instruction_selection: InstructionSelectionStrategy::TreeRewriting,
            register_allocation: RegisterAllocationStrategy::Greedy,
        };
        backends.insert("js".to_string(), js_backend);
        
        // バイトコードバックエンドの設定
        let bytecode_backend = CodeGenBackend {
            name: "bytecode".to_string(),
            architectures: ["vm"].iter().map(|s| s.to_string()).collect(),
            instruction_selection: InstructionSelectionStrategy::PatternMatching,
            register_allocation: RegisterAllocationStrategy::LinearScan,
        };
        backends.insert("bytecode".to_string(), bytecode_backend);
        
        // ABI定義の初期化
        let mut abi_definitions = HashMap::new();
        
        // システムV ABI
        let sysv_abi = ABIDefinition {
            name: "systemv".to_string(),
            // 呼び出し規約などの詳細は実際の実装で設定
        };
        abi_definitions.insert("systemv".to_string(), sysv_abi);
        
        // Windows x64 ABI
        let win64_abi = ABIDefinition {
            name: "win64".to_string(),
            // 呼び出し規約などの詳細は実際の実装で設定
        };
        abi_definitions.insert("win64".to_string(), win64_abi);
        
        // WebAssembly ABI
        let wasm_abi = ABIDefinition {
            name: "wasm".to_string(),
            // 呼び出し規約などの詳細は実際の実装で設定
        };
        abi_definitions.insert("wasm".to_string(), wasm_abi);
        
        // 言語定義から追加の設定を適用
        for (target_name, target_config) in &lang_def.semantics.target_specific {
            if supported_targets.contains(target_name) {
                // ターゲット固有の設定を適用
                if let Some(backend) = backends.get_mut(target_name) {
                    // ここでターゲット固有の設定を適用する
                    // 実際の実装ではtarget_configから設定を読み取る
                }
            }
        }
        
        CodeGenerator {
            supported_targets,
            backends,
            abi_definitions,
        }
    }
    
    /// 特定のターゲット向けのコード生成を実行
    pub fn generate_code(&self, target: &str, module: &crate::ir::Module) -> Result<Vec<u8>, String> {
        if !self.supported_targets.contains(target) {
            return Err(format!("サポートされていないターゲット: {}", target));
        }
        
        let backend = match self.backends.get(target) {
            Some(backend) => backend,
            None => return Err(format!("バックエンドが見つかりません: {}", target)),
        };
        
        // 実際のコード生成はターゲット固有のバックエンドに委譲
        // この実装はダミーで、実際にはバックエンドの実装を呼び出す
        Ok(Vec::new())
    }
    
    /// 新しいターゲットバックエンドを追加
    pub fn add_backend(&mut self, backend: CodeGenBackend) {
        self.supported_targets.insert(backend.name.clone());
        self.backends.insert(backend.name.clone(), backend);
    }
    
    /// 新しいABI定義を追加
    pub fn add_abi_definition(&mut self, abi: ABIDefinition) {
        self.abi_definitions.insert(abi.name.clone(), abi);
    }
}

/// コード生成バックエンド
#[derive(Debug, Clone)]
pub struct CodeGenBackend {
    /// バックエンドの名前
    pub name: String,
    
    /// サポートするアーキテクチャ
    pub architectures: HashSet<String>,
    
    /// 命令選択戦略
    pub instruction_selection: InstructionSelectionStrategy,
    
    /// レジスタ割り当て戦略
    pub register_allocation: RegisterAllocationStrategy,
}

/// 命令選択戦略
#[derive(Debug, Clone)]
pub enum InstructionSelectionStrategy {
    /// パターンマッチングベース
    PatternMatching,
    
    /// 木書き換えルール
    TreeRewriting,
    
    /// DAGベース
    DAGBased,
}

/// レジスタ割り当て戦略
#[derive(Debug, Clone)]
pub enum RegisterAllocationStrategy {
    /// リニアスキャン
    LinearScan,
    
    /// グラフ彩色
    GraphColoring,
    
    /// グリーディ
    Greedy,
}

/// ABI定義
#[derive(Debug, Clone)]
pub struct ABIDefinition {
    /// ABIの名前
    pub name: String,
    
    /// 呼び出し規約
    pub calling_convention: CallingConvention,
    
    /// データレイアウト
    pub data_layout: DataLayout,
}

/// 呼び出し規約
#[derive(Debug, Clone)]
pub struct CallingConvention {
    /// 引数の渡し方
    pub argument_passing: ArgumentPassing,
    
    /// 戻り値の受け取り方
    pub return_value: ReturnValueMethod,
    
    /// レジスタの利用規則
    pub register_usage: HashMap<String, RegisterUsage>,
}

/// 引数の渡し方
#[derive(Debug, Clone)]
pub enum ArgumentPassing {
    /// レジスタベース
    Registers(Vec<String>),
    
    /// スタックベース
    Stack,
    
    /// ハイブリッド
    Hybrid {
        register_count: usize,
        register_names: Vec<String>,
        stack_growth: StackGrowthDirection,
    },
}

/// スタックの成長方向
#[derive(Debug, Clone, Copy)]
pub enum StackGrowthDirection {
    /// 下向き
    Downward,
    
    /// 上向き
    Upward,
}

/// 戻り値の受け取り方
#[derive(Debug, Clone)]
pub enum ReturnValueMethod {
    /// レジスタ
    Register(String),
    
    /// レジスタの組
    Registers(Vec<String>),
    
    /// メモリ（ポインタ渡し）
    Memory,
}

/// レジスタの利用規則
#[derive(Debug, Clone)]
pub struct RegisterUsage {
    /// 呼び出し側保存（caller-saved）か呼び出し先保存（callee-saved）か
    pub preservation: RegisterPreservation,
    
    /// 特殊な用途があるか
    pub special_purpose: Option<String>,
}

/// レジスタの保存責任
#[derive(Debug, Clone, Copy)]
pub enum RegisterPreservation {
    /// 呼び出し側保存
    CallerSaved,
    
    /// 呼び出し先保存
    CalleeSaved,
}

/// データレイアウト
#[derive(Debug, Clone)]
pub struct DataLayout {
    /// エンディアン
    pub endianness: Endianness,
    
    /// アラインメント要件
    pub alignment_requirements: HashMap<String, usize>,
    
    /// 構造体のパディング規則
    pub struct_padding: StructPadding,
}

/// エンディアン
#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    /// ビッグエンディアン
    Big,
    
    /// リトルエンディアン
    Little,
}

/// 構造体のパディング規則
#[derive(Debug, Clone)]
pub enum StructPadding {
    /// 自然アラインメント
    Natural,
    
    /// パックド（詰め込み）
    Packed,
    
    /// カスタム
    Custom(Box<dyn Fn(Vec<Type>) -> Vec<usize> + Send + Sync>),
}

/// 評価戦略
#[derive(Debug, Clone, Copy)]
pub enum EvaluationStrategy {
    /// 正格評価
    StrictEvaluation,
    
    /// 遅延評価
    LazyEvaluation,
    
    /// 必要時評価
    CallByNeed,
}

/// 制御フロー定義
#[derive(Debug, Clone)]
pub struct ControlFlowDefinition {
    /// 条件分岐
    pub conditionals: Vec<ConditionalConstruct>,
    
    /// ループ構造
    pub loops: Vec<LoopConstruct>,
    
    /// 例外処理
    pub exception_handling: Option<ExceptionHandlingModel>,
    
    /// 継続（continuation）のサポート
    pub continuations: bool,
}

impl ControlFlowDefinition {
    /// 新しい制御フロー定義を作成
    pub fn new() -> Self {
        ControlFlowDefinition {
            conditionals: Vec::new(),
            loops: Vec::new(),
            exception_handling: None,
            continuations: false,
        }
    }
    
    /// 制御フロー定義を検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // 条件分岐構造の検証
        for conditional in &self.conditionals {
            if conditional.name.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    "条件分岐構造の名前が空です",
                    "条件分岐構造には有効な名前を設定してください",
                    None,
                    None,
                ));
            }
            
            // 構文ルールの検証
            if conditional.syntax.productions.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    format!("条件分岐構造 '{}' の構文ルールが定義されていません", conditional.name),
                    "少なくとも1つの構文プロダクションルールを定義してください",
                    None,
                    None,
                ));
            }
        }
        
        // ループ構造の検証
        for loop_construct in &self.loops {
            if loop_construct.name.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    "ループ構造の名前が空です",
                    "ループ構造には有効な名前を設定してください",
                    None,
                    None,
                ));
            }
            
            // 構文ルールの検証
            if loop_construct.syntax.productions.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    format!("ループ構造 '{}' の構文ルールが定義されていません", loop_construct.name),
                    "少なくとも1つの構文プロダクションルールを定義してください",
                    None,
                    None,
                ));
            }
            
            // 早期脱出と継続の両方が有効な場合の整合性チェック
            if loop_construct.early_exit && loop_construct.continuation {
                // 警告を追加（エラーではない）
                errors.push(ErrorDiagnostic::new_warning(
                    format!("ループ構造 '{}' は早期脱出と継続の両方をサポートしています", loop_construct.name),
                    "これらの機能の相互作用を慎重に設計してください",
                    None,
                    None,
                ));
            }
        }
        
        // 例外処理モデルの検証
        if let Some(exception_model) = &self.exception_handling {
            // 例外処理モデルと継続の組み合わせの検証
            if self.continuations && matches!(exception_model, ExceptionHandlingModel::ThrowCatch) {
                errors.push(ErrorDiagnostic::new_warning(
                    "継続と例外処理（ThrowCatch）の両方が有効になっています",
                    "これらの機能の相互作用は複雑になる可能性があります。慎重に設計してください",
                    None,
                    None,
                ));
            }
        }
        
        // 条件分岐構造とループ構造の名前の重複チェック
        let mut construct_names = HashSet::new();
        for conditional in &self.conditionals {
            if !construct_names.insert(&conditional.name) {
                errors.push(ErrorDiagnostic::new(
                    format!("制御フロー構造名 '{}' が重複しています", conditional.name),
                    "すべての制御フロー構造には一意の名前を付ける必要があります",
                    None,
                    None,
                ));
            }
        }
        
        for loop_construct in &self.loops {
            if !construct_names.insert(&loop_construct.name) {
                errors.push(ErrorDiagnostic::new(
                    format!("制御フロー構造名 '{}' が重複しています", loop_construct.name),
                    "すべての制御フロー構造には一意の名前を付ける必要があります",
                    None,
                    None,
                ));
            }
        }
        
        // エラーがあれば返す、なければOk
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 条件分岐構造を追加
    pub fn add_conditional(&mut self, conditional: ConditionalConstruct) -> Result<(), ErrorDiagnostic> {
        // 名前の重複チェック
        if self.conditionals.iter().any(|c| c.name == conditional.name) || 
           self.loops.iter().any(|l| l.name == conditional.name) {
            return Err(ErrorDiagnostic::new(
                format!("制御フロー構造名 '{}' が既に存在します", conditional.name),
                "一意の名前を使用してください",
                None,
                None,
            ));
        }
        
        // 構文ルールの検証
        if conditional.syntax.productions.is_empty() {
            return Err(ErrorDiagnostic::new(
                "構文ルールが定義されていません",
                "少なくとも1つの構文プロダクションルールを定義してください",
                None,
                None,
            ));
        }
        
        self.conditionals.push(conditional);
        Ok(())
    }
    
    /// ループ構造を追加
    pub fn add_loop(&mut self, loop_construct: LoopConstruct) -> Result<(), ErrorDiagnostic> {
        // 名前の重複チェック
        if self.conditionals.iter().any(|c| c.name == loop_construct.name) || 
           self.loops.iter().any(|l| l.name == loop_construct.name) {
            return Err(ErrorDiagnostic::new(
                format!("制御フロー構造名 '{}' が既に存在します", loop_construct.name),
                "一意の名前を使用してください",
                None,
                None,
            ));
        }
        
        // 構文ルールの検証
        if loop_construct.syntax.productions.is_empty() {
            return Err(ErrorDiagnostic::new(
                "構文ルールが定義されていません",
                "少なくとも1つの構文プロダクションルールを定義してください",
                None,
                None,
            ));
        }
        
        self.loops.push(loop_construct);
        Ok(())
    }
    
    /// 例外処理モデルを設定
    pub fn set_exception_handling(&mut self, model: ExceptionHandlingModel) {
        self.exception_handling = Some(model);
    }
    
    /// 継続サポートを設定
    pub fn set_continuations(&mut self, enabled: bool) {
        self.continuations = enabled;
    }
    
    /// 条件分岐構造を名前で取得
    pub fn get_conditional(&self, name: &str) -> Option<&ConditionalConstruct> {
        self.conditionals.iter().find(|c| c.name == name)
    }
    
    /// ループ構造を名前で取得
    pub fn get_loop(&self, name: &str) -> Option<&LoopConstruct> {
        self.loops.iter().find(|l| l.name == name)
    }
    
    /// 標準的な制御フロー定義を作成
    pub fn standard() -> Self {
        let mut def = Self::new();
        
        // if-else条件分岐を追加
        let if_else = ConditionalConstruct {
            name: "if-else".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("if (<expr>) <block> [else <block>]"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("条件式が真の場合は最初のブロックを実行し、偽の場合はelseブロックがあれば実行する"),
        };
        
        // switch-case条件分岐を追加
        let switch_case = ConditionalConstruct {
            name: "switch-case".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("switch (<expr>) { case <const_expr>: <stmt>* [default: <stmt>*] }"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("式の値に一致するcaseラベルに制御を移し、defaultがあれば一致するcaseがない場合に実行する"),
        };
        
        // for-loopを追加
        let for_loop = LoopConstruct {
            name: "for".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("for (<init>; <cond>; <update>) <block>"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("初期化を実行し、条件が真である間、ブロックと更新式を繰り返し実行する"),
            early_exit: true,
            continuation: false,
        };
        
        // while-loopを追加
        let while_loop = LoopConstruct {
            name: "while".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("while (<expr>) <block>"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("条件が真である間、ブロックを繰り返し実行する"),
            early_exit: true,
            continuation: false,
        };
        
        // do-while-loopを追加
        let do_while_loop = LoopConstruct {
            name: "do-while".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("do <block> while (<expr>);"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("ブロックを実行し、その後条件が真である間、ブロックを繰り返し実行する"),
            early_exit: true,
            continuation: false,
        };
        
        // foreach-loopを追加
        let foreach_loop = LoopConstruct {
            name: "foreach".to_string(),
            syntax: SyntaxRule {
                productions: vec![
                    ProductionRule::new("for (<type> <ident> in <expr>) <block>"),
                ],
                precedence: 0,
                associativity: None,
            },
            evaluation: SemanticsRule::new("イテラブルの各要素に対してブロックを実行する"),
            early_exit: true,
            continuation: false,
        };
        
        // 構造を追加
        let _ = def.add_conditional(if_else);
        let _ = def.add_conditional(switch_case);
        let _ = def.add_loop(for_loop);
        let _ = def.add_loop(while_loop);
        let _ = def.add_loop(do_while_loop);
        let _ = def.add_loop(foreach_loop);
        
        // 例外処理モデルを設定
        def.set_exception_handling(ExceptionHandlingModel::ThrowCatch);
        
        def
    }
}

/// 条件分岐構造
#[derive(Debug, Clone)]
pub struct ConditionalConstruct {
    /// 構造の名前
    pub name: String,
    
    /// 構文
    pub syntax: SyntaxRule,
    
    /// 評価ルール
    pub evaluation: SemanticsRule,
}

/// ループ構造
#[derive(Debug, Clone)]
pub struct LoopConstruct {
    /// 構造の名前
    pub name: String,
    
    /// 構文
    pub syntax: SyntaxRule,
    
    /// 評価ルール
    pub evaluation: SemanticsRule,
    
    /// 早期脱出のサポート
    pub early_exit: bool,
    
    /// 継続のサポート
    pub continuation: bool,
}

/// 例外処理モデル
#[derive(Debug, Clone)]
pub enum ExceptionHandlingModel {
    /// 戻り値ベース
    ReturnBased,
    
    /// 例外投げ/捕捉
    ThrowCatch,
    
    /// モナディック
    Monadic,
    
    /// エフェクトハンドラー
    EffectHandler,
}

/// 並行処理モデル
#[derive(Debug, Clone)]
pub enum ConcurrencyModel {
    /// スレッドベース
    ThreadBased {
        preemptive: bool,
        creation_mechanism: ThreadCreationMechanism,
    },
    
    /// イベントループ
    EventLoop {
        async_await: bool,
    },
    
    /// アクターモデル
    ActorModel {
        supervision: bool,
        message_passing: MessagePassingStyle,
    },
    
    /// CSP (Communicating Sequential Processes)
    CSP {
        channel_types: Vec<ChannelType>,
    },
    
    /// STM (Software Transactional Memory)
    STM,
}

/// スレッド作成メカニズム
#[derive(Debug, Clone)]
pub enum ThreadCreationMechanism {
    /// フォーク/ジョイン
    ForkJoin,
    
    /// スレッドプール
    ThreadPool,
    
    /// タスクベース
    TaskBased,
}

/// メッセージパッシングスタイル
#[derive(Debug, Clone)]
pub enum MessagePassingStyle {
    /// 同期
    Synchronous,
    
    /// 非同期
    Asynchronous,
    
    /// 選択的受信
    SelectiveReceive,
}

/// チャネルタイプ
#[derive(Debug, Clone)]
pub enum ChannelType {
    /// 同期
    Synchronous,
    
    /// 非同期（バッファあり）
    Buffered(usize),
    
    /// レンデズヴー
    Rendezvous,
}

/// エフェクトシステム
#[derive(Debug, Clone)]
pub enum EffectsSystem {
    /// 代数的エフェクト
    Algebraic {
        handlers: bool,
        resumable: bool,
    },
    
    /// モナディック
    Monadic {
        do_notation: bool,
    },
    
    /// 型エフェクト
    TypeEffects {
        region_based: bool,
    },
}

/// 型定義
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// 型の名前
    pub name: String,
    
    /// 型パラメータ
    pub type_parameters: Vec<TypeParameter>,
    
    /// 型の構造
    pub structure: TypeStructure,
    
    /// 型の制約
    pub constraints: Vec<TypeConstraint>,
}

/// 型パラメータ
#[derive(Debug, Clone)]
pub struct TypeParameter {
    /// パラメータの名前
    pub name: String,
    
    /// パラメータの制約
    pub constraints: Vec<TypeConstraint>,
    
    /// 変性（variance）
    pub variance: Variance,
}

/// 変性
#[derive(Debug, Clone, Copy)]
pub enum Variance {
    /// 共変
    Covariant,
    
    /// 反変
    Contravariant,
    
    /// 不変
    Invariant,
}

/// 型の構造
#[derive(Debug, Clone)]
pub enum TypeStructure {
    /// プリミティブ型
    Primitive {
        bit_width: usize,
        signed: bool,
    },
    
    /// 構造体型
    Struct {
        fields: Vec<FieldDefinition>,
        representation: StructRepresentation,
    },
    
    /// 和型 (enum, ADT)
    Sum {
        variants: Vec<VariantDefinition>,
        representation: EnumRepresentation,
    },
    
    /// 関数型
    Function {
        parameter_types: Vec<Type>,
        return_type: Box<Type>,
        effect_type: Option<Box<Type>>,
    },
    
    /// インターフェース型
    Interface {
        methods: Vec<MethodDefinition>,
    },
}

/// フィールド定義
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// フィールドの名前
    pub name: String,
    
    /// フィールドの型
    pub field_type: Type,
    
    /// フィールドの可視性
    pub visibility: Visibility,
    
    /// デフォルト値
    pub default_value: Option<String>,
}

/// 可視性
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// 公開
    Public,
    
    /// 保護
    Protected,
    
    /// 非公開
    Private,
    
    /// モジュール内
    Internal,
}

/// 構造体の表現
#[derive(Debug, Clone)]
pub enum StructRepresentation {
    /// デフォルト
    Default,
    
    /// パックド
    Packed(usize),
    
    /// タグ付き
    Tagged,
}

/// バリアント定義
#[derive(Debug, Clone)]
pub struct VariantDefinition {
    /// バリアントの名前
    pub name: String,
    
    /// バリアントの値
    pub value: Option<u64>,
    
    /// 関連データ
    pub associated_data: Option<Type>,
}

/// 列挙型の表現
#[derive(Debug, Clone)]
pub enum EnumRepresentation {
    /// タグ付き共用体
    Tagged,
    
    /// C風列挙型
    CEnum,
    
    /// 単純整数
    Primitive(usize),
    
    /// 文字列によるディスクリミネータ
    StringDiscriminated,
}

/// メソッド定義
#[derive(Debug, Clone)]
pub struct MethodDefinition {
    /// メソッドの名前
    pub name: String,
    
    /// メソッドの型
    pub method_type: FunctionType,
    
    /// メソッドの可視性
    pub visibility: Visibility,
    
    /// デフォルト実装
    pub default_implementation: Option<String>,
}

/// 関数型
#[derive(Debug, Clone)]
pub struct FunctionType {
    /// 型パラメータ
    pub type_parameters: Vec<TypeParameter>,
    
    /// パラメータ型
    pub parameter_types: Vec<Type>,
    
    /// 戻り値型
    pub return_type: Box<Type>,
    
    /// エフェクト型（オプション）
    pub effect_type: Option<Box<Type>>,
}

/// 型変換ルール
#[derive(Debug, Clone)]
pub struct ConversionRule {
    /// 変換元の型
    pub from_type: Type,
    
    /// 変換先の型
    pub to_type: Type,
    
    /// 変換の種類
    pub conversion_type: ConversionType,
    
    /// 変換の実装
    pub implementation: String,
}

/// 変換の種類
#[derive(Debug, Clone, Copy)]
pub enum ConversionType {
    /// 暗黙的変換
    Implicit,
    
    /// 明示的変換
    Explicit,
    
    /// 強制変換
    Cast,
}

/// 型推論ルール
#[derive(Debug, Clone)]
pub struct InferenceRule {
    /// ルールの名前
    pub name: String,
    
    /// 前提条件
    pub premises: Vec<TypeJudgment>,
    
    /// 結論
    pub conclusion: TypeJudgment,
}

/// 型判断
#[derive(Debug, Clone)]
pub struct TypeJudgment {
    /// コンテキスト
    pub context: TypeContext,
    
    /// 式
    pub expression: String,
    
    /// 型
    pub type_: Type,
}

/// 型コンテキスト
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// 型環境
    pub type_environment: HashMap<String, Type>,
    
    /// 制約
    pub constraints: Vec<TypeConstraint>,
}

/// サブタイプ関係
#[derive(Debug, Clone)]
pub struct SubtypeRelation {
    /// サブタイプ
    pub subtype: Type,
    
    /// スーパータイプ
    pub supertype: Type,
    
    /// サブタイプ証明
    pub proof: Option<String>,
}

/// モジュール定義
#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    /// モジュールの名前
    pub name: String,
    
    /// モジュールのバージョン
    pub version: String,
    
    /// エクスポートするシンボル
    pub exports: HashSet<String>,
    
    /// インポートするモジュール
    pub imports: Vec<ModuleImport>,
    
    /// モジュールの型定義
    pub type_definitions: Vec<TypeDefinition>,
    
    /// モジュールの関数定義
    pub function_definitions: Vec<FunctionDefinition>,
}

/// モジュールインポート
#[derive(Debug, Clone)]
pub struct ModuleImport {
    /// インポート元モジュール
    pub module_name: String,
    
    /// インポートするシンボル
    pub symbols: Vec<ImportedSymbol>,
    
    /// エイリアス
    pub alias: Option<String>,
}

/// インポートされたシンボル
#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    /// シンボル名
    pub name: String,
    
    /// エイリアス
    pub alias: Option<String>,
}

/// 関数定義
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// 関数名
    pub name: String,
    
    /// 型パラメータ
    pub type_parameters: Vec<TypeParameter>,
    
    /// パラメータ
    pub parameters: Vec<ParameterDefinition>,
    
    /// 戻り値型
    pub return_type: Type,
    
    /// 可視性
    pub visibility: Visibility,
    
    /// 実装
    pub implementation: Option<String>,
    
    /// 効果
    pub effects: Vec<Effect>,
    
    /// 属性
    pub attributes: Vec<Attribute>,
}

/// パラメータ定義
#[derive(Debug, Clone)]
pub struct ParameterDefinition {
    /// パラメータ名
    pub name: String,
    
    /// パラメータ型
    pub parameter_type: Type,
    
    /// デフォルト値
    pub default_value: Option<String>,
    
    /// パラメータスタイル
    pub style: ParameterStyle,
}

/// パラメータスタイル
#[derive(Debug, Clone, Copy)]
pub enum ParameterStyle {
    /// 値渡し
    ByValue,
    
    /// 参照渡し
    ByReference,
    
    /// 可変参照渡し
    ByMutableReference,
    
    /// ムーブ
    ByMove,
}

/// エフェクト
#[derive(Debug, Clone)]
pub struct Effect {
    /// エフェクト名
    pub name: String,
    
    /// エフェクトのパラメータ
    pub parameters: Vec<Type>,
}

/// 属性
#[derive(Debug, Clone)]
pub struct Attribute {
    /// 属性名
    pub name: String,
    
    /// 属性のパラメータ
    pub parameters: Vec<String>,
} 