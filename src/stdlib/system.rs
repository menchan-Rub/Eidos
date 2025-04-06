use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId, TypeKind, Field};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};

/// システムモジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 基本型の登録
    let int_type = Type::int();
    let bool_type = Type::bool();
    let string_type = Type::string();
    let unit_type = Type::unit();
    let string_array_type = Type::array(string_type.clone());
    
    // 環境変数関連の関数
    
    // System::getenv - 環境変数を取得
    registry.register_function(StdlibFunction::new(
        "getenv",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![("name".to_string(), string_type.id)],
        string_type.id,
        "指定された名前の環境変数の値を取得します。環境変数が存在しない場合は空文字列を返します。",
    ));
    
    // System::setenv - 環境変数を設定
    registry.register_function(StdlibFunction::new(
        "setenv",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![
            ("name".to_string(), string_type.id),
            ("value".to_string(), string_type.id),
        ],
        unit_type.id,
        "指定された名前と値で環境変数を設定します。",
    ));
    
    // System::unsetenv - 環境変数を削除
    registry.register_function(StdlibFunction::new(
        "unsetenv",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("name".to_string(), string_type.id)],
        unit_type.id,
        "指定された名前の環境変数を削除します。",
    ));
    
    // System::env_vars - すべての環境変数を取得
    registry.register_function(StdlibFunction::new(
        "env_vars",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_array_type.id,
        "すべての環境変数のキーと値のペアを「KEY=VALUE」形式の文字列配列として返します。",
    ));
    
    // プロセス関連の関数
    
    // System::exit - プロセスを終了
    registry.register_function(StdlibFunction::new(
        "exit",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("code".to_string(), int_type.id)],
        unit_type.id,
        "指定された終了コードでプロセスを終了します。",
    ));
    
    // System::execute - 外部コマンドを実行
    registry.register_function(StdlibFunction::new(
        "execute",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("command".to_string(), string_type.id)],
        int_type.id,
        "外部コマンドを実行し、終了コードを返します。",
    ));
    
    // System::execute_output - 外部コマンドを実行して出力を取得
    registry.register_function(StdlibFunction::new(
        "execute_output",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("command".to_string(), string_type.id)],
        string_type.id,
        "外部コマンドを実行し、標準出力を文字列として返します。",
    ));
    
    // System::execute_with_args - 引数付きで外部コマンドを実行
    registry.register_function(StdlibFunction::new(
        "execute_with_args",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![
            ("command".to_string(), string_type.id),
            ("args".to_string(), string_array_type.id),
        ],
        int_type.id,
        "外部コマンドを引数付きで実行し、終了コードを返します。",
    ));
    
    // System::pid - 現在のプロセスIDを取得
    registry.register_function(StdlibFunction::new(
        "pid",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "現在のプロセスIDを返します。",
    ));
    
    // System::ppid - 親プロセスIDを取得
    registry.register_function(StdlibFunction::new(
        "ppid",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "親プロセスIDを返します。",
    ));
    
    // オペレーティングシステム関連の関数
    
    // System::os_name - OSの名前を取得
    registry.register_function(StdlibFunction::new(
        "os_name",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "オペレーティングシステムの名前を返します。",
    ));
    
    // System::os_version - OSのバージョンを取得
    registry.register_function(StdlibFunction::new(
        "os_version",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "オペレーティングシステムのバージョンを返します。",
    ));
    
    // System::arch - マシンのアーキテクチャを取得
    registry.register_function(StdlibFunction::new(
        "arch",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "マシンのアーキテクチャを返します。",
    ));
    
    // System::hostname - ホスト名を取得
    registry.register_function(StdlibFunction::new(
        "hostname",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "コンピュータのホスト名を返します。",
    ));
    
    // System::username - ユーザー名を取得
    registry.register_function(StdlibFunction::new(
        "username",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "現在のユーザーの名前を返します。",
    ));
    
    // System::home_dir - ホームディレクトリを取得
    registry.register_function(StdlibFunction::new(
        "home_dir",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "現在のユーザーのホームディレクトリのパスを返します。",
    ));
    
    // System::current_dir - 現在の作業ディレクトリを取得
    registry.register_function(StdlibFunction::new(
        "current_dir",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "現在の作業ディレクトリのパスを返します。",
    ));
    
    // System::set_current_dir - 現在の作業ディレクトリを変更
    registry.register_function(StdlibFunction::new(
        "set_current_dir",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "現在の作業ディレクトリを指定されたパスに変更します。成功した場合はtrueを返します。",
    ));
    
    // システムリソース関連の関数
    
    // System::cpu_count - CPUコア数を取得
    registry.register_function(StdlibFunction::new(
        "cpu_count",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "システムのCPUコア数を返します。",
    ));
    
    // System::memory_usage - メモリ使用量を取得
    registry.register_function(StdlibFunction::new(
        "memory_usage",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "現在のプロセスのメモリ使用量をバイト単位で返します。",
    ));
    
    // System::total_memory - 合計物理メモリを取得
    registry.register_function(StdlibFunction::new(
        "total_memory",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "システムの合計物理メモリをバイト単位で返します。",
    ));
    
    // System::free_memory - 空き物理メモリを取得
    registry.register_function(StdlibFunction::new(
        "free_memory",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "システムの空き物理メモリをバイト単位で返します。",
    ));
    
    // System::uptime - システム稼働時間を取得
    registry.register_function(StdlibFunction::new(
        "uptime",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "システムの稼働時間を秒単位で返します。",
    ));
    
    // コマンドライン引数関連の関数
    
    // System::args - コマンドライン引数を取得
    registry.register_function(StdlibFunction::new(
        "args",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_array_type.id,
        "コマンドライン引数の配列を返します。最初の要素はプログラム名です。",
    ));
    
    // System::arg_count - コマンドライン引数の数を取得
    registry.register_function(StdlibFunction::new(
        "arg_count",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "コマンドライン引数の数を返します（プログラム名を含む）。",
    ));
    
    // System::arg - 指定されたインデックスのコマンドライン引数を取得
    registry.register_function(StdlibFunction::new(
        "arg",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![("index".to_string(), int_type.id)],
        string_type.id,
        "指定されたインデックスのコマンドライン引数を返します。インデックスが範囲外の場合は空文字列を返します。",
    ));
    
    // その他のシステム関数
    
    // System::random - ランダムな整数を生成
    registry.register_function(StdlibFunction::new(
        "random",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![
            ("min".to_string(), int_type.id),
            ("max".to_string(), int_type.id),
        ],
        int_type.id,
        "指定された範囲内のランダムな整数を生成します。",
    ));
    
    // System::random_seed - 乱数ジェネレータにシードを設定
    registry.register_function(StdlibFunction::new(
        "random_seed",
        StdlibModule::System,
        StdlibFunctionType::Effectful,
        vec![("seed".to_string(), int_type.id)],
        unit_type.id,
        "乱数ジェネレータにシードを設定します。",
    ));
    
    // System::uuid - UUIDを生成
    registry.register_function(StdlibFunction::new(
        "uuid",
        StdlibModule::System,
        StdlibFunctionType::Pure,
        vec![],
        string_type.id,
        "ランダムなUUID（汎用一意識別子）を生成して文字列として返します。",
    ));
    
    Ok(())
}

/// システム関数の実行
pub fn execute_function(function_name: &str, args: &[String]) -> Result<String> {
    match function_name {
        "getenv" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "getenv関数は1つの引数が必要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            let var_name = &args[0];
            match std::env::var(var_name) {
                Ok(value) => Ok(value),
                Err(_) => Ok("".to_string()), // 環境変数が存在しない場合は空文字列
            }
        }
        "pid" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "pid関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            Ok(std::process::id().to_string())
        }
        "os_name" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "os_name関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            Ok(std::env::consts::OS.to_string())
        }
        "arch" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "arch関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            Ok(std::env::consts::ARCH.to_string())
        }
        "current_dir" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "current_dir関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            match std::env::current_dir() {
                Ok(path) => Ok(path.to_string_lossy().to_string()),
                Err(e) => Err(EidosError::Runtime(format!("現在のディレクトリの取得に失敗しました: {}", e))),
            }
        }
        "args" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "args関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            let args: Vec<String> = std::env::args().collect();
            Ok(format!("[{}]", args.join(", ")))
        }
        "arg_count" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "arg_count関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            let count = std::env::args().count();
            Ok(count.to_string())
        }
        "exit" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "exit関数は1つの引数が必要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            let code = args[0].parse::<i32>().map_err(|_| {
                EidosError::Runtime("exit関数の引数は整数である必要があります。".to_string())
            })?;
            std::process::exit(code);
        }
        _ => Err(EidosError::Runtime(format!("システム関数 '{}' はネイティブ実装で提供されます", function_name)))
    }
} 