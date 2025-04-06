use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId, TypeKind, Field};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};

/// 入出力モジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 基本型の登録
    let int_type = Type::int();
    let bool_type = Type::bool();
    let string_type = Type::string();
    let unit_type = Type::unit();
    let bytes_type = Type::array(Type::byte());
    
    // File型の定義
    let file_type = Type::new(
        TypeKind::Struct {
            name: "File".to_string(),
            fields: vec![
                Field {
                    name: "path".to_string(),
                    field_type: string_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "is_open".to_string(),
                    field_type: bool_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "is_readable".to_string(),
                    field_type: bool_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "is_writable".to_string(),
                    field_type: bool_type.clone(),
                    is_public: false,
                },
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("io::File", file_type.clone());
    
    // Console関数の登録
    
    // Console::print - 標準出力に文字列を出力
    registry.register_function(StdlibFunction::new(
        "Console::print",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("message".to_string(), string_type.id)],
        unit_type.id,
        "標準出力に文字列を出力します。",
    ));
    
    // Console::println - 標準出力に文字列と改行を出力
    registry.register_function(StdlibFunction::new(
        "Console::println",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("message".to_string(), string_type.id)],
        unit_type.id,
        "標準出力に文字列と改行を出力します。",
    ));
    
    // Console::read_line - 標準入力から1行読み込む
    registry.register_function(StdlibFunction::new(
        "Console::read_line",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![],
        string_type.id,
        "標準入力から1行を読み込みます。",
    ));
    
    // Console::read_password - パスワードを読み込む（エコーバックなし）
    registry.register_function(StdlibFunction::new(
        "Console::read_password",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("prompt".to_string(), string_type.id)],
        string_type.id,
        "パスワードを読み込みます（エコーバックなし）。",
    ));
    
    // Console::clear - 画面をクリア
    registry.register_function(StdlibFunction::new(
        "Console::clear",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![],
        unit_type.id,
        "コンソール画面をクリアします。",
    ));
    
    // Console::set_color - 出力の色を設定
    registry.register_function(StdlibFunction::new(
        "Console::set_color",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("foreground".to_string(), string_type.id),
            ("background".to_string(), string_type.id),
        ],
        unit_type.id,
        "コンソール出力の色を設定します。",
    ));
    
    // Console::reset_color - 出力の色をリセット
    registry.register_function(StdlibFunction::new(
        "Console::reset_color",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![],
        unit_type.id,
        "コンソール出力の色をデフォルトにリセットします。",
    ));
    
    // File関数の登録
    
    // File::open - ファイルを開く
    registry.register_function(StdlibFunction::new(
        "File::open",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("path".to_string(), string_type.id),
            ("mode".to_string(), string_type.id),
        ],
        file_type.id,
        "ファイルを開きます。モードは 'r'（読み込み）, 'w'（書き込み）, 'a'（追加）のいずれかです。",
    ));
    
    // File::create - 新しいファイルを作成
    registry.register_function(StdlibFunction::new(
        "File::create",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("path".to_string(), string_type.id)],
        file_type.id,
        "新しいファイルを作成します。既に存在する場合は切り詰めます。",
    ));
    
    // File::close - ファイルを閉じる
    registry.register_function(StdlibFunction::new(
        "File::close",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        unit_type.id,
        "ファイルを閉じます。",
    ));
    
    // File::read - ファイルからデータを読み込む
    registry.register_function(StdlibFunction::new(
        "File::read",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        string_type.id,
        "ファイルの内容をすべて読み込みます。",
    ));
    
    // File::read_bytes - ファイルからバイトデータを読み込む
    registry.register_function(StdlibFunction::new(
        "File::read_bytes",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        bytes_type.id,
        "ファイルの内容をバイト配列として読み込みます。",
    ));
    
    // File::read_line - ファイルから1行読み込む
    registry.register_function(StdlibFunction::new(
        "File::read_line",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        string_type.id,
        "ファイルから1行を読み込みます。",
    ));
    
    // File::read_lines - ファイルから全行を読み込む
    registry.register_function(StdlibFunction::new(
        "File::read_lines",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        Type::array(string_type.clone()).id,
        "ファイルからすべての行を読み込み、配列として返します。",
    ));
    
    // File::write - ファイルに文字列を書き込む
    registry.register_function(StdlibFunction::new(
        "File::write",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("file".to_string(), file_type.id),
            ("content".to_string(), string_type.id),
        ],
        unit_type.id,
        "ファイルに文字列を書き込みます。",
    ));
    
    // File::write_bytes - ファイルにバイトデータを書き込む
    registry.register_function(StdlibFunction::new(
        "File::write_bytes",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("file".to_string(), file_type.id),
            ("data".to_string(), bytes_type.id),
        ],
        unit_type.id,
        "ファイルにバイト配列を書き込みます。",
    ));
    
    // File::write_line - ファイルに行を書き込む
    registry.register_function(StdlibFunction::new(
        "File::write_line",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("file".to_string(), file_type.id),
            ("line".to_string(), string_type.id),
        ],
        unit_type.id,
        "ファイルに文字列と改行を書き込みます。",
    ));
    
    // File::append - ファイルに文字列を追加
    registry.register_function(StdlibFunction::new(
        "File::append",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("file".to_string(), file_type.id),
            ("content".to_string(), string_type.id),
        ],
        unit_type.id,
        "ファイルに文字列を追加します。",
    ));
    
    // File::flush - ファイルバッファをフラッシュ
    registry.register_function(StdlibFunction::new(
        "File::flush",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("file".to_string(), file_type.id)],
        unit_type.id,
        "ファイルバッファをフラッシュします。",
    ));
    
    // File::seek - ファイル内の位置を移動
    registry.register_function(StdlibFunction::new(
        "File::seek",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![
            ("file".to_string(), file_type.id),
            ("position".to_string(), int_type.id),
        ],
        unit_type.id,
        "ファイル内の位置を移動します。",
    ));
    
    // File::position - 現在のファイル位置を取得
    registry.register_function(StdlibFunction::new(
        "File::position",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("file".to_string(), file_type.id)],
        int_type.id,
        "現在のファイル位置を取得します。",
    ));
    
    // File::length - ファイルサイズを取得
    registry.register_function(StdlibFunction::new(
        "File::length",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("file".to_string(), file_type.id)],
        int_type.id,
        "ファイルサイズを取得します。",
    ));
    
    // ディレクトリ関連の関数登録
    
    // Directory::create - ディレクトリを作成
    registry.register_function(StdlibFunction::new(
        "Directory::create",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "ディレクトリを作成します。成功した場合はtrueを返します。",
    ));
    
    // Directory::remove - ディレクトリを削除
    registry.register_function(StdlibFunction::new(
        "Directory::remove",
        StdlibModule::IO,
        StdlibFunctionType::Effectful,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "ディレクトリを削除します。成功した場合はtrueを返します。",
    ));
    
    // Directory::exists - ディレクトリが存在するかを確認
    registry.register_function(StdlibFunction::new(
        "Directory::exists",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "指定したパスにディレクトリが存在するかどうかを確認します。",
    ));
    
    // Directory::list - ディレクトリ内のファイル一覧を取得
    registry.register_function(StdlibFunction::new(
        "Directory::list",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        Type::array(string_type.clone()).id,
        "ディレクトリ内のファイルとサブディレクトリの一覧を取得します。",
    ));
    
    // Path関連の関数登録
    
    // Path::exists - パスが存在するかを確認
    registry.register_function(StdlibFunction::new(
        "Path::exists",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "指定したパスが存在するかどうかを確認します。",
    ));
    
    // Path::is_file - パスがファイルかどうかを確認
    registry.register_function(StdlibFunction::new(
        "Path::is_file",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "指定したパスがファイルかどうかを確認します。",
    ));
    
    // Path::is_directory - パスがディレクトリかどうかを確認
    registry.register_function(StdlibFunction::new(
        "Path::is_directory",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        bool_type.id,
        "指定したパスがディレクトリかどうかを確認します。",
    ));
    
    // Path::join - パスを結合
    registry.register_function(StdlibFunction::new(
        "Path::join",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![
            ("base".to_string(), string_type.id),
            ("path".to_string(), string_type.id),
        ],
        string_type.id,
        "2つのパスを結合します。",
    ));
    
    // Path::parent - 親ディレクトリのパスを取得
    registry.register_function(StdlibFunction::new(
        "Path::parent",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        string_type.id,
        "パスの親ディレクトリのパスを取得します。",
    ));
    
    // Path::file_name - ファイル名を取得
    registry.register_function(StdlibFunction::new(
        "Path::file_name",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        string_type.id,
        "パスからファイル名を取得します。",
    ));
    
    // Path::extension - ファイル拡張子を取得
    registry.register_function(StdlibFunction::new(
        "Path::extension",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        string_type.id,
        "パスからファイル拡張子を取得します。",
    ));
    
    // Path::absolute - 絶対パスを取得
    registry.register_function(StdlibFunction::new(
        "Path::absolute",
        StdlibModule::IO,
        StdlibFunctionType::Pure,
        vec![("path".to_string(), string_type.id)],
        string_type.id,
        "相対パスから絶対パスを取得します。",
    ));
    
    Ok(())
}

/// 入出力関数の実行
pub fn execute_function(function_name: &str, args: &[String]) -> Result<String> {
    match function_name {
        // Console関数
        "Console::print" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "Console::print は1つの引数が必要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            print!("{}", args[0]);
            Ok("".to_string())
        }
        "Console::println" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "Console::println は1つの引数が必要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            println!("{}", args[0]);
            Ok("".to_string())
        }
        _ => Err(EidosError::Runtime(format!("入出力関数 '{}' はネイティブ実装で提供されます", function_name)))
    }
} 