// 構文定義のためのDSL
// Eidosが他の汎用言語の構文を定義するための基盤

use std::collections::{HashMap, HashSet};
use crate::error::ErrorDiagnostic;
use crate::core::language_def::{Lexer, TokenDefinition, Parser};

/// 構文定義
#[derive(Debug, Clone)]
pub struct SyntaxDefinition {
    /// トークン定義
    pub tokens: Vec<TokenDefinition>,
    
    /// キーワード
    pub keywords: HashSet<String>,
    
    /// 文法ルール
    pub rules: Vec<ProductionRule>,
    
    /// 演算子の優先順位とアソシアティビティ
    pub operators: OperatorTable,
    
    /// 文脈自由文法の種類
    pub grammar_type: GrammarType,
    
    /// エラーリカバリー戦略
    pub error_recovery: ErrorRecoveryStrategy,
    
    /// 構文エラーのメッセージ
    pub error_messages: HashMap<String, String>,
}

impl SyntaxDefinition {
    /// 新しい構文定義を作成
    pub fn new() -> Self {
        SyntaxDefinition {
            tokens: Vec::new(),
            keywords: HashSet::new(),
            rules: Vec::new(),
            operators: OperatorTable::new(),
            grammar_type: GrammarType::LL(1),
            error_recovery: ErrorRecoveryStrategy::Panic,
            error_messages: HashMap::new(),
        }
    }
    
    /// トークン定義を追加
    pub fn add_token(&mut self, name: &str, pattern: &str, priority: u32) {
        self.tokens.push(TokenDefinition {
            name: name.to_string(),
            pattern: pattern.to_string(),
            priority,
        });
    }
    
    /// キーワードを追加
    pub fn add_keyword(&mut self, keyword: &str) {
        self.keywords.insert(keyword.to_string());
    }
    
    /// 文法ルールを追加
    pub fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule);
    }
    
    /// 演算子を追加
    pub fn add_operator(&mut self, op: &str, precedence: u32, associativity: Associativity) {
        self.operators.add_operator(op, precedence, associativity);
    }
    
    /// 構文定義を検証
    pub fn validate(&self) -> Result<(), Vec<ErrorDiagnostic>> {
        let mut errors = Vec::new();
        
        // トークン定義の検証
        for token in &self.tokens {
            if token.name.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    "Token name cannot be empty".to_string(),
                    "SyntaxDefinition".to_string(),
                    None,
                ));
            }
            
            if token.pattern.is_empty() {
                errors.push(ErrorDiagnostic::new(
                    format!("Token pattern for '{}' cannot be empty", token.name),
                    "SyntaxDefinition".to_string(),
                    None,
                ));
            }
        }
        
        // 文法ルールの検証
        let mut non_terminals = HashSet::new();
        
        // 非終端記号を収集
        for rule in &self.rules {
            non_terminals.insert(rule.name.clone());
        }
        
        // 参照されるがルールが無い非終端記号を検出
        for rule in &self.rules {
            for symbol in &rule.rhs {
                match symbol {
                    Symbol::NonTerminal(name) => {
                        if !non_terminals.contains(name) {
                            errors.push(ErrorDiagnostic::new(
                                format!("Reference to undefined non-terminal: '{}'", name),
                                "SyntaxDefinition".to_string(),
                                None,
                            ));
                        }
                    },
                    _ => {},
                }
            }
        }
        
        // 開始記号が定義されているか確認
        if !self.rules.iter().any(|r| r.name == "start") {
            errors.push(ErrorDiagnostic::new(
                "Grammar must define a 'start' rule".to_string(),
                "SyntaxDefinition".to_string(),
                None,
            ));
        }
        
        // 左再帰の検出（LL文法の場合）
        if let GrammarType::LL(_) = self.grammar_type {
            for rule in &self.rules {
                if self.has_left_recursion(rule, &non_terminals, &mut HashSet::new()) {
                    errors.push(ErrorDiagnostic::new(
                        format!("Left recursion detected in rule: '{}'", rule.name),
                        "SyntaxDefinition".to_string(),
                        None,
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 左再帰を検出
    fn has_left_recursion(&self, rule: &ProductionRule, non_terminals: &HashSet<String>, visited: &mut HashSet<String>) -> bool {
        if visited.contains(&rule.name) {
            return false;
        }
        
        visited.insert(rule.name.clone());
        
        for alternative in &rule.alternatives {
            if let Some(first_symbol) = alternative.first() {
                match first_symbol {
                    Symbol::NonTerminal(name) => {
                        // 直接左再帰
                        if *name == rule.name {
                            return true;
                        }
                        
                        // 間接左再帰
                        if let Some(referenced_rule) = self.rules.iter().find(|r| r.name == *name) {
                            if self.has_left_recursion(referenced_rule, non_terminals, visited) {
                                return true;
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
        
        visited.remove(&rule.name);
        false
    }
    
    /// 構文定義から字句解析器を生成
    pub fn generate_lexer(&self) -> Lexer {
        let mut skip_tokens = HashSet::new();
        
        // コメントや空白などのスキップするトークンを収集
        for token in &self.tokens {
            if token.name == "WHITESPACE" || token.name == "COMMENT" {
                skip_tokens.insert(token.name.clone());
            }
        }
        
        Lexer {
            token_definitions: self.tokens.clone(),
            skip_tokens,
        }
    }
    
    /// 構文定義からパーサーを生成
    pub fn generate_parser(&self) -> Parser {
        Parser {
            grammar_rules: self.rules.clone(),
            parser_type: match self.grammar_type {
                GrammarType::LL(k) => {
                    if k == 1 {
                        crate::core::language_def::ParserType::PredictiveLL(1)
                    } else {
                        crate::core::language_def::ParserType::RecursiveDescent
                    }
                },
                GrammarType::LR(lr_type) => {
                    match lr_type {
                        LRType::SLR => crate::core::language_def::ParserType::LR(crate::core::language_def::LRVariant::SLR),
                        LRType::LALR => crate::core::language_def::ParserType::LR(crate::core::language_def::LRVariant::LALR),
                        LRType::CLR => crate::core::language_def::ParserType::LR(crate::core::language_def::LRVariant::CanonicalLR),
                    }
                },
            },
            error_recovery: match self.error_recovery {
                ErrorRecoveryStrategy::Panic => crate::core::language_def::ErrorRecoveryStrategy::Panic,
                ErrorRecoveryStrategy::PhraseLevel => crate::core::language_def::ErrorRecoveryStrategy::PhraseLevel,
                ErrorRecoveryStrategy::ErrorProductions => crate::core::language_def::ErrorRecoveryStrategy::ErrorProduction,
            },
        }
    }
}

/// 文法の種類
#[derive(Debug, Clone)]
pub enum GrammarType {
    /// LL(k)文法
    LL(usize),
    
    /// LR系文法
    LR(LRType),
}

/// LR文法の種類
#[derive(Debug, Clone)]
pub enum LRType {
    /// シンプルLR
    SLR,
    
    /// 先読みLR
    LALR,
    
    /// 正準LR
    CLR,
}

/// エラーリカバリー戦略
#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    /// パニックモード（エラー発生時に停止）
    Panic,
    
    /// フレーズレベルの回復
    PhraseLevel,
    
    /// エラー生成ルール
    ErrorProductions,
}

/// 演算子テーブル
#[derive(Debug, Clone)]
pub struct OperatorTable {
    /// 演算子の優先順位
    pub precedence: HashMap<String, u32>,
    
    /// 演算子の結合性
    pub associativity: HashMap<String, Associativity>,
}

impl OperatorTable {
    /// 新しい演算子テーブルを作成
    pub fn new() -> Self {
        OperatorTable {
            precedence: HashMap::new(),
            associativity: HashMap::new(),
        }
    }
    
    /// 演算子を追加
    pub fn add_operator(&mut self, op: &str, precedence: u32, associativity: Associativity) {
        self.precedence.insert(op.to_string(), precedence);
        self.associativity.insert(op.to_string(), associativity);
    }
}

/// 演算子の結合性
#[derive(Debug, Clone, Copy)]
pub enum Associativity {
    /// 左結合
    Left,
    
    /// 右結合
    Right,
    
    /// 結合なし
    None,
}

/// 生成規則
#[derive(Debug, Clone)]
pub struct ProductionRule {
    /// 規則の名前（非終端記号）
    pub name: String,
    
    /// 代替生成規則
    pub alternatives: Vec<Vec<Symbol>>,
    
    /// 規則の意味アクション（オプション）
    pub semantic_action: Option<String>,
    
    /// フォロー集合（LL(1)パーサーの生成に使用）
    pub follow_set: Option<HashSet<String>>,
}

impl ProductionRule {
    /// 新しい生成規則を作成
    pub fn new(name: &str) -> Self {
        ProductionRule {
            name: name.to_string(),
            alternatives: Vec::new(),
            semantic_action: None,
            follow_set: None,
        }
    }
    
    /// 代替生成規則を追加
    pub fn add_alternative(&mut self, symbols: Vec<Symbol>) {
        self.alternatives.push(symbols);
    }
    
    /// 意味アクションを設定
    pub fn set_semantic_action(&mut self, action: &str) {
        self.semantic_action = Some(action.to_string());
    }
}

/// シンボル（文法記号）
#[derive(Debug, Clone)]
pub enum Symbol {
    /// 終端記号（トークン）
    Terminal(String),
    
    /// 非終端記号
    NonTerminal(String),
    
    /// イプシロン（空文字列）
    Epsilon,
    
    /// 繰り返し（0回以上）
    ZeroOrMore(Box<Symbol>),
    
    /// 繰り返し（1回以上）
    OneOrMore(Box<Symbol>),
    
    /// オプション（0回または1回）
    Optional(Box<Symbol>),
    
    /// 選択（どれか1つ）
    Choice(Vec<Symbol>),
    
    /// グループ化
    Group(Vec<Symbol>),
}

/// 構文木ノード
#[derive(Debug, Clone)]
pub struct SyntaxNode {
    /// ノードの種類
    pub kind: String,
    
    /// ノードの値
    pub value: Option<String>,
    
    /// 子ノード
    pub children: Vec<SyntaxNode>,
    
    /// ノードの位置情報
    pub location: Option<SourceLocation>,
}

/// ソースコード内の位置
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    /// 開始行
    pub start_line: usize,
    
    /// 開始列
    pub start_column: usize,
    
    /// 終了行
    pub end_line: usize,
    
    /// 終了列
    pub end_column: usize,
}

/// 構文定義DSL
pub mod dsl {
    use super::*;
    
    /// 終端記号を作成
    pub fn terminal(name: &str) -> Symbol {
        Symbol::Terminal(name.to_string())
    }
    
    /// 非終端記号を作成
    pub fn non_terminal(name: &str) -> Symbol {
        Symbol::NonTerminal(name.to_string())
    }
    
    /// イプシロン（空文字列）を作成
    pub fn epsilon() -> Symbol {
        Symbol::Epsilon
    }
    
    /// 0回以上の繰り返しを作成
    pub fn zero_or_more(symbol: Symbol) -> Symbol {
        Symbol::ZeroOrMore(Box::new(symbol))
    }
    
    /// 1回以上の繰り返しを作成
    pub fn one_or_more(symbol: Symbol) -> Symbol {
        Symbol::OneOrMore(Box::new(symbol))
    }
    
    /// オプションを作成
    pub fn optional(symbol: Symbol) -> Symbol {
        Symbol::Optional(Box::new(symbol))
    }
    
    /// 選択を作成
    pub fn choice(symbols: Vec<Symbol>) -> Symbol {
        Symbol::Choice(symbols)
    }
    
    /// グループを作成
    pub fn group(symbols: Vec<Symbol>) -> Symbol {
        Symbol::Group(symbols)
    }
    
    /// 規則を作成
    pub fn rule(name: &str, alternatives: Vec<Vec<Symbol>>) -> ProductionRule {
        let mut rule = ProductionRule::new(name);
        for alt in alternatives {
            rule.add_alternative(alt);
        }
        rule
    }
    
    /// 意味アクション付きの規則を作成
    pub fn rule_with_action(name: &str, alternatives: Vec<Vec<Symbol>>, action: &str) -> ProductionRule {
        let mut rule = rule(name, alternatives);
        rule.set_semantic_action(action);
        rule
    }
}

/// 構文定義DSLのマクロ
#[macro_export]
macro_rules! syntax {
    // 構文定義全体
    ($name:ident {
        $($content:tt)*
    }) => {
        {
            let mut syntax_def = SyntaxDefinition::new();
            syntax_def.grammar_type = GrammarType::LL(1);
            $($crate::syntax_item!(syntax_def, $content);)*
            syntax_def
        }
    };
}

/// 構文アイテムのマクロヘルパー
#[macro_export]
macro_rules! syntax_item {
    // トークン定義
    ($syntax:ident, token $name:ident = $pattern:expr, $priority:expr;) => {
        $syntax.add_token(stringify!($name), $pattern, $priority);
    };
    
    // キーワード定義
    ($syntax:ident, keyword $name:ident;) => {
        $syntax.add_keyword(stringify!($name));
    };
    
    // 演算子定義
    ($syntax:ident, operator $op:expr, $precedence:expr, $associativity:expr;) => {
        $syntax.add_operator($op, $precedence, $associativity);
    };
    
    // 規則定義
    ($syntax:ident, rule $name:ident = $($rhs:tt)*;) => {
        {
            let mut rule = ProductionRule::new(stringify!($name));
            $crate::syntax_rhs!(rule, $($rhs)*);
            $syntax.add_rule(rule);
        }
    };
    
    // 意味アクション付き規則定義
    ($syntax:ident, rule $name:ident = $($rhs:tt)* => $action:expr;) => {
        {
            let mut rule = ProductionRule::new(stringify!($name));
            $crate::syntax_rhs!(rule, $($rhs)*);
            rule.set_semantic_action($action);
            $syntax.add_rule(rule);
        }
    };
    
    // グラマーの種類指定
    ($syntax:ident, grammar_type = $type:ident $(( $($args:expr),* ))? ;) => {
        $syntax.grammar_type = GrammarType::$type($($(($args)),*)?);
    };
    
    // エラーリカバリー戦略指定
    ($syntax:ident, error_recovery = $strategy:ident;) => {
        $syntax.error_recovery = ErrorRecoveryStrategy::$strategy;
    };
}

/// 構文右辺のマクロヘルパー
#[macro_export]
macro_rules! syntax_rhs {
    // 終端記号
    ($rule:ident, $terminal:literal $($rest:tt)*) => {
        let mut symbols = vec![Symbol::Terminal($terminal.to_string())];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // 非終端記号
    ($rule:ident, $non_terminal:ident $($rest:tt)*) => {
        let mut symbols = vec![Symbol::NonTerminal(stringify!($non_terminal).to_string())];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // イプシロン
    ($rule:ident, ε $($rest:tt)*) => {
        let mut symbols = vec![Symbol::Epsilon];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // 代替規則（|で区切られたもの）
    ($rule:ident, $($first:tt)+ | $($second:tt)+) => {
        $crate::syntax_rhs!($rule, $($first)+);
        $crate::syntax_rhs!($rule, $($second)+);
    };
    
    // 繰り返し（0回以上）
    ($rule:ident, ($($inner:tt)+)* $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        let mut symbols = vec![Symbol::ZeroOrMore(Box::new(Symbol::Group(inner_symbols)))];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // 繰り返し（1回以上）
    ($rule:ident, ($($inner:tt)+)+ $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        let mut symbols = vec![Symbol::OneOrMore(Box::new(Symbol::Group(inner_symbols)))];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // オプション
    ($rule:ident, ($($inner:tt)+)? $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        let mut symbols = vec![Symbol::Optional(Box::new(Symbol::Group(inner_symbols)))];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
    
    // グループ
    ($rule:ident, ( $($inner:tt)+ ) $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        let mut symbols = vec![Symbol::Group(inner_symbols)];
        $crate::syntax_rhs_rest!(symbols, $($rest)*);
        $rule.add_alternative(symbols);
    };
}

/// 構文右辺の残りのマクロヘルパー
#[macro_export]
macro_rules! syntax_rhs_rest {
    // 終端記号
    ($symbols:ident, $terminal:literal $($rest:tt)*) => {
        $symbols.push(Symbol::Terminal($terminal.to_string()));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // 非終端記号
    ($symbols:ident, $non_terminal:ident $($rest:tt)*) => {
        $symbols.push(Symbol::NonTerminal(stringify!($non_terminal).to_string()));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // イプシロン
    ($symbols:ident, ε $($rest:tt)*) => {
        $symbols.push(Symbol::Epsilon);
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // 繰り返し（0回以上）
    ($symbols:ident, ($($inner:tt)+)* $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        $symbols.push(Symbol::ZeroOrMore(Box::new(Symbol::Group(inner_symbols))));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // 繰り返し（1回以上）
    ($symbols:ident, ($($inner:tt)+)+ $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        $symbols.push(Symbol::OneOrMore(Box::new(Symbol::Group(inner_symbols))));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // オプション
    ($symbols:ident, ($($inner:tt)+)? $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        $symbols.push(Symbol::Optional(Box::new(Symbol::Group(inner_symbols))));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // グループ
    ($symbols:ident, ( $($inner:tt)+ ) $($rest:tt)*) => {
        let mut inner_symbols = Vec::new();
        $crate::syntax_rhs_inner!(inner_symbols, $($inner)+);
        $symbols.push(Symbol::Group(inner_symbols));
        $crate::syntax_rhs_rest!($symbols, $($rest)*);
    };
    
    // 残りが無い場合
    ($symbols:ident, ) => {};
}

/// 構文右辺の内部のマクロヘルパー
#[macro_export]
macro_rules! syntax_rhs_inner {
    // 終端記号
    ($symbols:ident, $terminal:literal $($rest:tt)*) => {
        $symbols.push(Symbol::Terminal($terminal.to_string()));
        $crate::syntax_rhs_inner!($symbols, $($rest)*);
    };
    
    // 非終端記号
    ($symbols:ident, $non_terminal:ident $($rest:tt)*) => {
        $symbols.push(Symbol::NonTerminal(stringify!($non_terminal).to_string()));
        $crate::syntax_rhs_inner!($symbols, $($rest)*);
    };
    
    // イプシロン
    ($symbols:ident, ε $($rest:tt)*) => {
        $symbols.push(Symbol::Epsilon);
        $crate::syntax_rhs_inner!($symbols, $($rest)*);
    };
    
    // 選択
    ($symbols:ident, $($first:tt)+ | $($second:tt)+) => {
        let mut choices = Vec::new();
        
        let mut first_symbols = Vec::new();
        $crate::syntax_rhs_inner!(first_symbols, $($first)+);
        choices.push(Symbol::Group(first_symbols));
        
        let mut second_symbols = Vec::new();
        $crate::syntax_rhs_inner!(second_symbols, $($second)+);
        choices.push(Symbol::Group(second_symbols));
        
        $symbols.push(Symbol::Choice(choices));
    };
    
    // 残りが無い場合
    ($symbols:ident, ) => {};
} 