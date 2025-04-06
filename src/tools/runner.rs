use std::fs;
use std::path::{Path, PathBuf};

use log::{info, debug};

use crate::core::{Result, EidosError, SourceLocation};
use crate::frontend::{Lexer, Parser, TypeChecker, SemanticAnalyzer};
use crate::core::eir::{Module, ModuleBuilder};
use crate::backend::{Backend, CodegenOptions, OutputFormat, Target, BackendFactory};
use crate::backend::wasm::WasmRuntime;

/// Eidosファイルを実行
pub fn run_file(file: &Path, args: Vec<String>) -> Result<()> {
    info!("ファイルを実行中: {}", file.display());
    
    // 引数を表示
    if !args.is_empty() {
        debug!("実行引数: {:?}", args);
    }
    
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
    
    // EIR（Eidos中間表現）に変換
    debug!("中間表現に変換中");
    let mut module_builder = ModuleBuilder::new(file.file_name().unwrap().to_string_lossy().to_string());
    let module = module_builder.build_from_ast(&typed_ast)?;
    
    // WebAssemblyバックエンドでコンパイル
    debug!("WebAssemblyにコンパイル中");
    let backend_factory = BackendFactory::new();
    let backend = backend_factory.create_backend(Target::Wasm)?;
    
    let options = CodegenOptions {
        format: OutputFormat::WASM,
        optimization_level: 2, // 最適化レベル（0-3）
        debug_info: true,
    };
    
    // コードの生成
    let wasm_bytes = backend.compile(&module, &options)?;
    
    // WebAssemblyモジュールを実行
    debug!("WebAssemblyモジュールを実行中");
    let mut runtime = WasmRuntime::new()?;
    runtime.run_module(&wasm_bytes)?;
    
    info!("実行が正常に終了しました");
    
    Ok(())
} 