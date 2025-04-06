use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Write};

use log::{info, debug, error};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::core::{Result, EidosError, SourceLocation};
use crate::frontend::{Lexer, Parser, TypeChecker, SemanticAnalyzer};

/// REPLを起動
pub fn start_repl(preload: Option<Vec<PathBuf>>) -> Result<()> {
    info!("Eidos REPL を起動中");
    
    println!("Eidos REPL v0.1.0");
    println!("'exit' または Ctrl+D で終了");
    
    // 履歴機能付きの入力エディタを初期化
    let mut rl = Editor::<()>::new().expect("Rustylineの初期化に失敗しました");
    
    // 履歴ファイルがあれば読み込む
    if let Err(err) = rl.load_history(".eidos_history") {
        debug!("履歴ファイルの読み込みに失敗: {}", err);
        // エラーは無視して続行
    }
    
    // 事前ロードファイルの処理
    if let Some(files) = preload {
        for file in files {
            if let Err(e) = preload_file(&file) {
                eprintln!("ファイルのプリロードに失敗: {}: {}", file.display(), e);
            }
        }
    }
    
    // REPLのメインループ
    loop {
        // プロンプトを表示して入力を受け付ける
        match rl.readline(">>> ") {
            Ok(line) => {
                // 空行はスキップ
                if line.trim().is_empty() {
                    continue;
                }
                
                // 'exit' で終了
                if line.trim() == "exit" {
                    println!("Eidosを終了します");
                    break;
                }
                
                // 入力を履歴に追加
                rl.add_history_entry(&line);
                
                // 入力を評価
                match evaluate_input(&line) {
                    Ok(result) => {
                        println!("{}", result);
                    },
                    Err(e) => {
                        eprintln!("エラー: {}", e);
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - 入力をクリアして続行
                println!("入力をクリアしました");
                continue;
            },
            Err(ReadlineError::Eof) => {
                // Ctrl+D - 終了
                println!("Eidosを終了します");
                break;
            },
            Err(err) => {
                // その他のエラー
                error!("入力エラー: {}", err);
                eprintln!("入力エラー: {}", err);
                break;
            }
        }
    }
    
    // 履歴を保存
    if let Err(err) = rl.save_history(".eidos_history") {
        debug!("履歴ファイルの保存に失敗: {}", err);
        // エラーは無視して続行
    }
    
    info!("REPL終了");
    Ok(())
}

/// 入力を評価
fn evaluate_input(input: &str) -> Result<String> {
    // 仮想ファイルパス
    let file_path = PathBuf::from("<repl>");
    
    // 字句解析
    let mut lexer = Lexer::new(input, file_path.clone());
    let tokens = lexer.tokenize()?;
    
    // トークンの表示（デバッグ用）
    debug!("トークン: {:?}", tokens);
    
    // 構文解析
    let mut parser = Parser::new(tokens, file_path.clone());
    let ast = parser.parse()?;
    
    // AST表示（デバッグ用）
    debug!("AST: {:?}", ast);
    
    // 意味解析
    let mut analyzer = SemanticAnalyzer::new();
    let analyzed_ast = analyzer.analyze(ast)?;
    
    // 型チェック
    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.check(analyzed_ast)?;
    
    // REPLの実装はまだ完全ではないため、簡易的な結果を返す
    Ok("評価が完了しました (具体的な値の表示は未実装)".to_string())
}

/// ファイルをプリロード
fn preload_file(file: &Path) -> Result<()> {
    info!("ファイルをプリロード中: {}", file.display());
    
    // ファイルを読み込み
    let source = fs::read_to_string(file).map_err(|e| {
        EidosError::IO(e)
    })?;
    
    // プリロードファイルの処理は単純に評価と同じ
    evaluate_input(&source)?;
    
    Ok(())
} 