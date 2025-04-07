use std::path::PathBuf;
use thiserror::Error;
use miette::{Diagnostic, SourceSpan, MietteError, Report};
use log::error;

/// Eidos言語の処理中に発生する可能性のあるすべてのエラー
#[derive(Error, Debug, Diagnostic)]
pub enum EidosError {
    #[error("字句解析エラー: {0}")]
    #[diagnostic(code(eidos::lexer))]
    LexerError(String),

    #[error("構文解析エラー: {0}")]
    #[diagnostic(code(eidos::parser))]
    ParserError(String),

    #[error("型エラー: {0}")]
    #[diagnostic(code(eidos::type_checker))]
    TypeError(String),

    #[error("名前解決エラー: {0}")]
    #[diagnostic(code(eidos::name_resolver))]
    NameResolutionError(String),

    #[error("意味解析エラー: {0}")]
    #[diagnostic(code(eidos::semantic_analyzer))]
    SemanticError(String),

    #[error("バックエンドエラー: {0}")]
    #[diagnostic(code(eidos::backend))]
    BackendError(String),

    #[error("IOエラー: {0}")]
    #[diagnostic(code(eidos::io))]
    IOError(#[from] std::io::Error),

    #[error("DSLエラー: {0}")]
    #[diagnostic(code(eidos::dsl))]
    DSLError(String),

    #[error("内部エラー: {0}")]
    #[diagnostic(code(eidos::internal))]
    InternalError(String),
    
    #[error("実行時エラー: {0}")]
    #[diagnostic(code(eidos::runtime))]
    RuntimeError(String),
    
    #[error("機能未実装: {0}")]
    #[diagnostic(code(eidos::not_implemented))]
    NotImplemented(String),
    
    #[error("環境エラー: {0}")]
    #[diagnostic(code(eidos::environment))]
    EnvironmentError(String),
    
    #[error("複合エラー: 複数の問題が検出されました")]
    #[diagnostic(code(eidos::multiple))]
    MultipleErrors(Vec<Box<EidosError>>),
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
    
    /// ファイル名と行番号を含む文字列表現
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", 
                self.file.display(), 
                self.line, 
                self.column)
    }
}

/// ソースコード位置情報付きエラー
#[derive(Error, Debug, Diagnostic)]
#[error("{kind}")]
pub struct SourceError {
    #[source_code]
    pub source: String,
    
    #[label("この位置で発生")]
    pub span: SourceSpan,
    
    pub kind: EidosError,
    
    pub file: Option<PathBuf>,
    
    pub line: usize,
    
    pub column: usize,
}

impl SourceError {
    /// 新しいソースエラーを作成
    pub fn new(
        kind: EidosError,
        source: String,
        span: (usize, usize),
        file: Option<PathBuf>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            kind,
            source,
            span: span.into(),
            file,
            line,
            column,
        }
    }
    
    /// ファイル名を表示形式で取得
    pub fn file_display(&self) -> String {
        match &self.file {
            Some(path) => path.display().to_string(),
            None => "<unknown>".to_string(),
        }
    }
    
    /// エラーメッセージを詳細な位置情報付きで取得
    pub fn with_location(&self) -> String {
        format!(
            "{}:{}: {}",
            self.file_display(),
            self.line,
            self.kind
        )
    }
    
    /// miette形式のレポートに変換
    pub fn to_report(&self) -> Report {
        Report::new(self.clone())
    }
    
    /// エラーをログに記録しつつレポートとして返す
    pub fn log_and_report(&self) -> Report {
        error!("{}", self.with_location());
        self.to_report()
    }
}

/// エラーを収集するためのコレクタ
#[derive(Default, Debug)]
pub struct ErrorCollector {
    errors: Vec<Box<EidosError>>,
}

impl ErrorCollector {
    /// 新しいエラーコレクタを作成
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
    
    /// エラーを追加
    pub fn add(&mut self, error: EidosError) {
        self.errors.push(Box::new(error));
    }
    
    /// エラーが存在するかチェック
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// エラーの数を取得
    pub fn count(&self) -> usize {
        self.errors.len()
    }
    
    /// エラーを複合エラーとして返す
    pub fn into_error(self) -> Option<EidosError> {
        if self.errors.is_empty() {
            None
        } else if self.errors.len() == 1 {
            // 単一エラーの場合はそのまま返す
            Some(*self.errors.into_iter().next().unwrap())
        } else {
            // 複数エラーの場合は MultipleErrors にまとめる
            Some(EidosError::MultipleErrors(self.errors))
        }
    }
    
    /// すべてのエラーを取得
    pub fn all_errors(&self) -> &[Box<EidosError>] {
        &self.errors
    }
}

/// Result型のエイリアス
pub type EidosResult<T> = Result<T, EidosError>;

/// ソース位置情報付きのResult型
pub type SourceResult<T> = Result<T, SourceError>; 