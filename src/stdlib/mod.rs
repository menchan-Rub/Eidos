use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;

use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId};

pub mod math;
pub mod string;
pub mod collections;
pub mod io;
pub mod time;
pub mod system;

/// 標準ライブラリ関数の実行タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibFunctionType {
    /// 副作用のない純粋関数
    Pure,
    /// 副作用のある関数
    Effectful,
}

/// 標準ライブラリモジュール
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StdlibModule {
    /// 数学関数
    Math,
    /// 文字列操作
    String,
    /// コレクションデータ構造
    Collections,
    /// 入出力処理
    IO,
    /// 時間関連
    Time,
    /// システム関連
    System,
}

impl StdlibModule {
    /// モジュール名を取得
    pub fn name(&self) -> &'static str {
        match self {
            StdlibModule::Math => "math",
            StdlibModule::String => "string",
            StdlibModule::Collections => "collections",
            StdlibModule::IO => "io",
            StdlibModule::Time => "time",
            StdlibModule::System => "system",
        }
    }
}

/// 標準ライブラリ関数
#[derive(Debug, Clone)]
pub struct StdlibFunction {
    /// 関数名
    pub name: String,
    /// モジュール
    pub module: StdlibModule,
    /// 関数タイプ
    pub fn_type: StdlibFunctionType,
    /// 引数（名前と型ID）
    pub args: Vec<(String, TypeId)>,
    /// 戻り値の型ID
    pub return_type: TypeId,
    /// 関数の説明
    pub description: String,
}

impl StdlibFunction {
    /// 新しい標準ライブラリ関数を作成
    pub fn new(
        name: &str,
        module: StdlibModule,
        fn_type: StdlibFunctionType,
        args: Vec<(String, TypeId)>,
        return_type: TypeId,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            module,
            fn_type,
            args,
            return_type,
            description: description.to_string(),
        }
    }

    /// 完全修飾名を取得 (モジュール::関数名)
    pub fn full_name(&self) -> String {
        format!("{}::{}", self.module.name(), self.name)
    }
}

/// 標準ライブラリレジストリ
#[derive(Debug, Default)]
pub struct StdlibRegistry {
    /// 型のマップ
    pub types: HashMap<String, Type>,
    /// 関数のマップ
    pub functions: HashMap<String, StdlibFunction>,
}

impl StdlibRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// グローバルレジストリを取得
    pub fn global() -> Arc<RwLock<Self>> {
        STDLIB_REGISTRY.clone()
    }

    /// 標準ライブラリを初期化
    pub fn initialize() -> Result<()> {
        let mut registry = Self::global().write().unwrap();
        
        // 各モジュールを初期化
        math::initialize(&mut registry)?;
        string::initialize(&mut registry)?;
        collections::initialize(&mut registry)?;
        io::initialize(&mut registry)?;
        time::initialize(&mut registry)?;
        system::initialize(&mut registry)?;
        
        Ok(())
    }

    /// 型を登録
    pub fn register_type(&mut self, name: &str, type_def: Type) {
        self.types.insert(name.to_string(), type_def);
    }

    /// 関数を登録
    pub fn register_function(&mut self, function: StdlibFunction) {
        self.functions.insert(function.name.clone(), function);
    }

    /// 型を名前で取得
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    /// 関数を名前で取得
    pub fn get_function(&self, name: &str) -> Option<&StdlibFunction> {
        self.functions.get(name)
    }

    /// 指定されたモジュールの関数一覧を取得
    pub fn get_module_functions(&self, module: StdlibModule) -> Vec<&StdlibFunction> {
        self.functions
            .values()
            .filter(|f| f.module == module)
            .collect()
    }

    /// 標準ライブラリ関数を実行
    pub fn execute_function(&self, function_name: &str, args: &[String]) -> Result<String> {
        // モジュール名と関数名に分割
        let parts: Vec<&str> = function_name.split("::").collect();
        if parts.len() != 2 {
            return Err(EidosError::Runtime(format!(
                "無効な関数名: {}（モジュール::関数名の形式が必要）",
                function_name
            )));
        }

        let module_name = parts[0];
        let fn_name = parts[1];

        // モジュールに基づいて関数を実行
        match module_name {
            "math" => math::execute_function(fn_name, args),
            "string" => string::execute_function(fn_name, args),
            "collections" => collections::execute_function(fn_name, args),
            "io" => io::execute_function(fn_name, args),
            "time" => time::execute_function(fn_name, args),
            "system" => system::execute_function(fn_name, args),
            _ => Err(EidosError::Runtime(format!("不明なモジュール: {}", module_name))),
        }
    }
}

// グローバルレジストリのシングルトンインスタンス
lazy_static! {
    static ref STDLIB_REGISTRY: Arc<RwLock<StdlibRegistry>> = Arc::new(RwLock::new(StdlibRegistry::new()));
} 