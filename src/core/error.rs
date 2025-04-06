use std::path::PathBuf;
use thiserror::Error;

/// Eidos言語の処理中に発生する可能性のあるすべてのエラー
#[derive(Error, Debug)]
pub enum EidosError {
    #[error("入出力エラー: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("字句解析エラー: {message} ({file}:{line}:{column})")]
    Lexer {
        message: String,
        file: PathBuf,
        line: usize,
        column: usize,
    },
    
    #[error("構文解析エラー: {message} ({file}:{line}:{column})")]
    Parser {
        message: String,
        file: PathBuf,
        line: usize,
        column: usize,
    },
    
    #[error("型エラー: {message} ({file}:{line}:{column})")]
    Type {
        message: String,
        file: PathBuf,
        line: usize,
        column: usize,
    },
    
    #[error("意味解析エラー: {message} ({file}:{line}:{column})")]
    Semantic {
        message: String,
        file: PathBuf,
        line: usize,
        column: usize,
    },
    
    #[error("コード生成エラー: {0}")]
    CodeGen(String),
    
    #[error("DSL拡張エラー: {message} ({dsl_name})")]
    DSL {
        message: String,
        dsl_name: String,
    },
    
    #[error("ランタイムエラー: {0}")]
    Runtime(String),
    
    #[error("内部エラー: {0}")]
    Internal(String),
}

/// エラー位置情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize, column: usize, length: usize) -> Self {
        Self { file, line, column, length }
    }
    
    pub fn unknown() -> Self {
        Self {
            file: PathBuf::from("<unknown>"),
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

/// 結果型の短縮形
pub type Result<T> = std::result::Result<T, EidosError>; 