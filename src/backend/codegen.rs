use std::path::Path;
use std::collections::HashMap;

use log::{info, debug, error};

use crate::core::{Result, EidosError};
use crate::core::eir::{Module, Function, FunctionId, BlockId, InstructionId, RegisterId, Instruction, Operand, Literal};
use crate::core::types::{Type, TypeId};
use crate::core::symbol::SymbolId;

use super::llvm::LLVMBackend;
use super::wasm::WasmBackend;

/// 出力コードの形式
pub enum OutputFormat {
    /// 機械語バイナリ
    Binary,
    /// LLVMビットコード
    Bitcode,
    /// LLVM IR (人間可読テキスト形式)
    LLVMIR,
    /// WebAssembly
    Wasm,
    /// WebAssembly テキスト形式
    Wat,
}

/// コード生成のターゲット
pub enum Target {
    /// ホストマシンのネイティブコード
    Native,
    /// 特定のターゲットトリプル
    Triple(String),
    /// WebAssembly
    Wasm,
}

/// コード生成オプション
pub struct CodegenOptions {
    /// 出力形式
    pub format: OutputFormat,
    /// 対象ターゲット
    pub target: Target,
    /// 最適化レベル（0-3）
    pub opt_level: u8,
    /// デバッグ情報を含める
    pub debug_info: bool,
    /// LTO (Link Time Optimization) を有効にする
    pub lto: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Binary,
            target: Target::Native,
            opt_level: 2,
            debug_info: false,
            lto: false,
        }
    }
}

/// コード生成器のトレイト
pub trait Backend {
    /// バックエンドの名前
    fn name(&self) -> &str;
    
    /// コンパイル
    fn compile(&self, module: &Module, options: &CodegenOptions) -> Result<Vec<u8>>;
    
    /// 関数宣言
    fn declare_function(&mut self, name: &str, params: &[Type], return_type: &Type) -> Result<()>;
    
    /// シンボルをグローバル変数として宣言
    fn declare_global(&mut self, name: &str, ty: &Type, initializer: Option<&Literal>) -> Result<()>;
}

/// コード生成器
pub struct CodeGenerator {
    /// 使用するバックエンド
    backend: Box<dyn Backend>,
    /// 型情報のキャッシュ
    type_cache: HashMap<TypeId, Type>,
    /// レジスタと型のマッピング
    register_types: HashMap<RegisterId, TypeId>,
    /// シンボルと名前のマッピング
    symbol_names: HashMap<SymbolId, String>,
}

impl CodeGenerator {
    /// LLVMバックエンドを使用するコード生成器を作成
    pub fn new_llvm() -> Self {
        Self {
            backend: Box::new(LLVMBackend::new()),
            type_cache: HashMap::new(),
            register_types: HashMap::new(),
            symbol_names: HashMap::new(),
        }
    }
    
    /// WebAssemblyバックエンドを使用するコード生成器を作成
    pub fn new_wasm() -> Self {
        Self {
            backend: Box::new(WasmBackend::new()),
            type_cache: HashMap::new(),
            register_types: HashMap::new(),
            symbol_names: HashMap::new(),
        }
    }
    
    /// コンパイル実行
    pub fn compile(&mut self, module: &Module, options: &CodegenOptions, output_path: &Path) -> Result<()> {
        info!("コード生成を開始: {}", module.name);
        
        // バックエンドを使用してコンパイル
        let code = self.backend.compile(module, options)?;
        
        // 出力ファイルに書き込み
        std::fs::write(output_path, code).map_err(|e| {
            EidosError::IO(e)
        })?;
        
        info!("コード生成が完了しました: {}", output_path.display());
        Ok(())
    }
    
    /// モジュールを処理
    fn process_module(&mut self, module: &Module) -> Result<()> {
        // グローバル変数を宣言
        for (symbol_id, (type_id, initializer)) in &module.globals {
            let symbol_name = self.get_symbol_name(*symbol_id, module);
            let ty = module.type_map.get(type_id).unwrap();
            
            self.backend.declare_global(&symbol_name, ty, initializer.as_ref())?;
        }
        
        // 関数を処理
        for (function_id, function) in &module.functions {
            self.process_function(function, module)?;
        }
        
        Ok(())
    }
    
    /// 関数を処理
    fn process_function(&mut self, function: &Function, module: &Module) -> Result<()> {
        debug!("関数を処理中: {}", function.name);
        
        // 関数パラメータの型を取得
        let mut param_types = Vec::new();
        for (_, type_id) in &function.params {
            if let Some(ty) = module.type_map.get(type_id) {
                param_types.push(ty.clone());
            } else {
                return Err(EidosError::CodeGen(format!(
                    "関数 '{}' のパラメータの型が見つかりません", function.name
                )));
            }
        }
        
        // 戻り値の型を取得
        let return_type = module.type_map.get(&function.return_type).ok_or_else(|| {
            EidosError::CodeGen(format!(
                "関数 '{}' の戻り値の型が見つかりません", function.name
            ))
        })?;
        
        // 関数を宣言
        self.backend.declare_function(&function.name, &param_types, return_type)?;
        
        // ブロックを処理（ここでは省略）
        
        Ok(())
    }
    
    /// シンボルIDから名前を取得（もしくは生成）
    fn get_symbol_name(&mut self, symbol_id: SymbolId, module: &Module) -> String {
        if let Some(name) = self.symbol_names.get(&symbol_id) {
            return name.clone();
        }
        
        // シンボル名がキャッシュになければ生成
        let name = format!("sym_{}", symbol_id);
        self.symbol_names.insert(symbol_id, name.clone());
        name
    }
} 