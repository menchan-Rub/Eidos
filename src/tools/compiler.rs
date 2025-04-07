use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::time::Instant;
use log::{info, debug, warn, error};
use colored::Colorize;

use crate::core::error::{EidosError, SourceError, ErrorCollector};
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::frontend::semantic_analyzer::SemanticAnalyzer;
use crate::frontend::type_checker::TypeChecker;
use crate::core::ast::Program;
use crate::backend::codegen::CodeGenerator;

/// コンパイルオプション
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// 最適化レベル (0-3)
    pub opt_level: u8,
    /// デバッグ情報を含めるか
    pub debug_info: bool,
    /// 出力ファイルのパス
    pub output_path: Option<PathBuf>,
    /// 実行後に削除するか
    pub run_after_compile: bool,
    /// 詳細表示モード
    pub verbose: bool,
    /// ターゲットバックエンド
    pub target: CompileTarget,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            opt_level: 2,
            debug_info: false,
            output_path: None,
            run_after_compile: false,
            verbose: false,
            target: CompileTarget::Native,
        }
    }
}

/// コンパイルターゲット
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileTarget {
    /// ネイティブバイナリ (デフォルト)
    Native,
    /// LLVM IR
    LLVM,
    /// WebAssembly
    WASM,
    /// C言語コード
    C,
}

/// コンパイル結果の統計情報
#[derive(Debug)]
pub struct CompileStats {
    /// コンパイル時間 (ミリ秒)
    pub compile_time_ms: u128,
    /// 生成されたコードサイズ (バイト)
    pub code_size: usize,
    /// 警告の数
    pub warnings: usize,
    /// エラーの数
    pub errors: usize,
    /// ASTノードの数
    pub ast_nodes: usize,
}

/// ファイルをコンパイル
pub fn compile_file(file: &Path, opt_level: u8, output: Option<PathBuf>) -> Result<()> {
    let options = CompileOptions {
        opt_level,
        output_path: output,
        ..Default::default()
    };
    
    compile_with_options(file, &options)
}

/// 詳細なオプションでファイルをコンパイル
pub fn compile_with_options(file: &Path, options: &CompileOptions) -> Result<()> {
    let start_time = Instant::now();
    info!("コンパイル開始: {}", file.display());
    debug!("コンパイルオプション: {:?}", options);
    
    // エラーコレクタ
    let mut error_collector = ErrorCollector::new();
    
    // ソースコードの読み込み
    let source = std::fs::read_to_string(file)
        .context(format!("ファイルの読み込みに失敗しました: {}", file.display()))?;
    
    // コンパイルプロセス
    let ast = match parse_source(&source, file, &mut error_collector) {
        Ok(ast) => ast,
        Err(e) => {
            error!("構文解析エラー: {}", e);
            return Err(e.into());
        }
    };
    
    // 意味解析
    let analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&ast) {
        error_collector.add(e);
    }
    
    // 型検査
    let type_checker = TypeChecker::new();
    if let Err(e) = type_checker.check_program(&ast) {
        error_collector.add(e);
    }
    
    // エラーがある場合は終了
    if error_collector.has_errors() {
        if let Some(error) = error_collector.into_error() {
            error!("コンパイルエラー: {}", error);
            return Err(error.into());
        }
    }
    
    // コード生成
    let output_path = options.output_path.clone().unwrap_or_else(|| {
        let stem = file.file_stem().unwrap_or_default();
        PathBuf::from(stem)
    });
    
    let generator = CodeGenerator::new(options.opt_level);
    generator.generate(&ast, &output_path)
        .context("コード生成に失敗しました")?;
    
    // 統計情報
    let elapsed = start_time.elapsed();
    info!("コンパイル完了: {} ({:?})", output_path.display(), elapsed);
    
    if options.verbose {
        let stats = CompileStats {
            compile_time_ms: elapsed.as_millis(),
            code_size: std::fs::metadata(&output_path).map(|m| m.len() as usize).unwrap_or(0),
            warnings: 0, // TODO: 警告カウント
            errors: 0,
            ast_nodes: count_ast_nodes(&ast),
        };
        
        print_compile_stats(&stats);
    }
    
    Ok(())
}

/// ファイルの型チェックのみ行う
pub fn typecheck_file(file: &Path) -> Result<()> {
    info!("型チェック開始: {}", file.display());
    
    // エラーコレクタ
    let mut error_collector = ErrorCollector::new();
    
    // ソースコードの読み込み
    let source = std::fs::read_to_string(file)
        .context(format!("ファイルの読み込みに失敗しました: {}", file.display()))?;
    
    // 構文解析
    let ast = match parse_source(&source, file, &mut error_collector) {
        Ok(ast) => ast,
        Err(e) => {
            error!("構文解析エラー: {}", e);
            return Err(e.into());
        }
    };
    
    // 型検査
    let type_checker = TypeChecker::new();
    if let Err(e) = type_checker.check_program(&ast) {
        error_collector.add(e);
    }
    
    // エラーがある場合は終了
    if error_collector.has_errors() {
        if let Some(error) = error_collector.into_error() {
            error!("型エラー: {}", error);
            return Err(error.into());
        }
    }
    
    info!("型チェック完了: {}", file.display());
    Ok(())
}

/// ソースコードを構文解析
fn parse_source(source: &str, file_path: &Path, error_collector: &mut ErrorCollector) -> Result<Program> {
    // 字句解析
    let lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            error_collector.add(e);
            return Err(EidosError::LexerError("字句解析に失敗しました".to_string()).into());
        }
    };
    
    // 構文解析
    let parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => Ok(program),
        Err(e) => {
            error_collector.add(e);
            Err(EidosError::ParserError("構文解析に失敗しました".to_string()).into())
        }
    }
}

/// ASTノードの数をカウント
fn count_ast_nodes(program: &Program) -> usize {
    // 簡易的な実装 - 実際にはすべてのノードを再帰的にカウントする
    program.statements.len()
}

/// コンパイル統計情報を表示
fn print_compile_stats(stats: &CompileStats) {
    println!("{}", "==== コンパイル統計 ====".green().bold());
    println!("コンパイル時間: {}ms", stats.compile_time_ms);
    println!("生成コードサイズ: {}バイト", stats.code_size);
    println!("ASTノード数: {}", stats.ast_nodes);
    println!("警告: {}", stats.warnings);
    println!("エラー: {}", stats.errors);
    println!("{}", "========================".green().bold());
} 