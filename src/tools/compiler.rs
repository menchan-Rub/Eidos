use std::fs;
use std::path::{Path, PathBuf};

use log::{info, debug};

use crate::core::{Result, EidosError, SourceLocation};
use crate::frontend::{Lexer, Parser, TypeChecker, SemanticAnalyzer};

/// ファイルをコンパイル
pub fn compile_file(file: &Path, opt_level: u8, output: Option<PathBuf>) -> Result<()> {
    info!("ファイルをコンパイル中: {}", file.display());
    
    // 出力ファイル名を決定
    let output = output.unwrap_or_else(|| {
        let mut output = file.to_path_buf();
        output.set_extension("out");
        output
    });
    
    // ファイルを読み込み
    debug!("ソースファイルを読み込み中");
    let source = fs::read_to_string(file).map_err(|e| {
        EidosError::IO(e)
    })?;
    
    // 字句解析
    debug!("字句解析を実行中");
    let mut lexer = Lexer::new(&source, file.to_path_buf());
    let tokens = lexer.tokenize()?;
    
    // 構文解析
    debug!("構文解析を実行中");
    let mut parser = Parser::new(tokens, file.to_path_buf());
    let ast = parser.parse()?;
    
    // 意味解析
    debug!("意味解析を実行中");
    let mut analyzer = SemanticAnalyzer::new();
    let analyzed_ast = analyzer.analyze(ast)?;
    
    // 型チェック
    debug!("型チェックを実行中");
    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.check(analyzed_ast)?;
    
    // ここではまだコード生成が未実装のため、簡易的に成功を返す
    info!("コンパイル成功: {}", output.display());
    Ok(())
}

/// ファイルの型チェックのみ実行
pub fn typecheck_file(file: &Path) -> Result<()> {
    info!("ファイルの型チェックを実行中: {}", file.display());
    
    // ファイルを読み込み
    debug!("ソースファイルを読み込み中");
    let source = fs::read_to_string(file).map_err(|e| {
        EidosError::IO(e)
    })?;
    
    // 字句解析
    debug!("字句解析を実行中");
    let mut lexer = Lexer::new(&source, file.to_path_buf());
    let tokens = lexer.tokenize()?;
    
    // 構文解析
    debug!("構文解析を実行中");
    let mut parser = Parser::new(tokens, file.to_path_buf());
    let ast = parser.parse()?;
    
    // 意味解析
    debug!("意味解析を実行中");
    let mut analyzer = SemanticAnalyzer::new();
    let analyzed_ast = analyzer.analyze(ast)?;
    
    // 型チェック
    debug!("型チェックを実行中");
    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.check(analyzed_ast)?;
    
    info!("型チェック成功");
    Ok(())
} 