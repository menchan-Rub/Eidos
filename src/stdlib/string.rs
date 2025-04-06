use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};

/// 文字列モジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 型の登録
    let string_type = Type::string();
    let int_type = Type::int();
    let bool_type = Type::bool();
    let char_type = Type::char();
    let array_string_type = Type::array(Type::string());
    
    // 関数の登録
    
    // length - 文字列の長さを取得
    registry.register_function(StdlibFunction::new(
        "length",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        int_type.id,
        "文字列の長さを文字数で返します。",
    ));
    
    // is_empty - 文字列が空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_empty",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        bool_type.id,
        "文字列が空文字列かどうかを返します。",
    ));
    
    // concat - 文字列を連結
    registry.register_function(StdlibFunction::new(
        "concat",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("a".to_string(), string_type.id),
            ("b".to_string(), string_type.id),
        ],
        string_type.id,
        "2つの文字列を連結します。",
    ));
    
    // substr - 部分文字列を取得
    registry.register_function(StdlibFunction::new(
        "substr",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("start".to_string(), int_type.id),
            ("length".to_string(), int_type.id),
        ],
        string_type.id,
        "文字列の一部を取得します。startは開始位置（0から始まる）、lengthは取得する文字数です。",
    ));
    
    // at - 指定位置の文字を取得
    registry.register_function(StdlibFunction::new(
        "at",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("index".to_string(), int_type.id),
        ],
        char_type.id,
        "文字列の指定位置にある文字を返します。indexは0から始まります。",
    ));
    
    // contains - 文字列に部分文字列が含まれているかを確認
    registry.register_function(StdlibFunction::new(
        "contains",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("substr".to_string(), string_type.id),
        ],
        bool_type.id,
        "文字列に部分文字列が含まれているかどうかを返します。",
    ));
    
    // starts_with - 文字列が指定された接頭辞で始まるかを確認
    registry.register_function(StdlibFunction::new(
        "starts_with",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("prefix".to_string(), string_type.id),
        ],
        bool_type.id,
        "文字列が指定された接頭辞で始まるかどうかを返します。",
    ));
    
    // ends_with - 文字列が指定された接尾辞で終わるかを確認
    registry.register_function(StdlibFunction::new(
        "ends_with",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("suffix".to_string(), string_type.id),
        ],
        bool_type.id,
        "文字列が指定された接尾辞で終わるかどうかを返します。",
    ));
    
    // index_of - 部分文字列の位置を検索
    registry.register_function(StdlibFunction::new(
        "index_of",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("substr".to_string(), string_type.id),
        ],
        int_type.id,
        "文字列内の部分文字列の位置を返します。見つからない場合は-1を返します。",
    ));
    
    // last_index_of - 部分文字列の最後の位置を検索
    registry.register_function(StdlibFunction::new(
        "last_index_of",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("substr".to_string(), string_type.id),
        ],
        int_type.id,
        "文字列内の部分文字列の最後の位置を返します。見つからない場合は-1を返します。",
    ));
    
    // replace - 部分文字列を置換
    registry.register_function(StdlibFunction::new(
        "replace",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("from".to_string(), string_type.id),
            ("to".to_string(), string_type.id),
        ],
        string_type.id,
        "文字列内のすべての部分文字列を別の文字列に置き換えます。",
    ));
    
    // replace_first - 最初の部分文字列を置換
    registry.register_function(StdlibFunction::new(
        "replace_first",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("from".to_string(), string_type.id),
            ("to".to_string(), string_type.id),
        ],
        string_type.id,
        "文字列内の最初の部分文字列を別の文字列に置き換えます。",
    ));
    
    // to_upper - 文字列を大文字に変換
    registry.register_function(StdlibFunction::new(
        "to_upper",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        string_type.id,
        "文字列をすべて大文字に変換します。",
    ));
    
    // to_lower - 文字列を小文字に変換
    registry.register_function(StdlibFunction::new(
        "to_lower",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        string_type.id,
        "文字列をすべて小文字に変換します。",
    ));
    
    // trim - 文字列の前後の空白を削除
    registry.register_function(StdlibFunction::new(
        "trim",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        string_type.id,
        "文字列の前後の空白を削除します。",
    ));
    
    // trim_start - 文字列の先頭の空白を削除
    registry.register_function(StdlibFunction::new(
        "trim_start",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        string_type.id,
        "文字列の先頭の空白を削除します。",
    ));
    
    // trim_end - 文字列の末尾の空白を削除
    registry.register_function(StdlibFunction::new(
        "trim_end",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        string_type.id,
        "文字列の末尾の空白を削除します。",
    ));
    
    // split - 文字列を分割
    registry.register_function(StdlibFunction::new(
        "split",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("delimiter".to_string(), string_type.id),
        ],
        array_string_type.id,
        "文字列を指定された区切り文字で分割し、文字列の配列を返します。",
    ));
    
    // join - 文字列の配列を結合
    registry.register_function(StdlibFunction::new(
        "join",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("strings".to_string(), array_string_type.id),
            ("delimiter".to_string(), string_type.id),
        ],
        string_type.id,
        "文字列の配列を指定された区切り文字で結合します。",
    ));
    
    // repeat - 文字列を指定回数繰り返す
    registry.register_function(StdlibFunction::new(
        "repeat",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("count".to_string(), int_type.id),
        ],
        string_type.id,
        "文字列を指定された回数だけ繰り返します。",
    ));
    
    // char_at - 文字列の指定位置の文字コードを取得
    registry.register_function(StdlibFunction::new(
        "char_at",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![
            ("str".to_string(), string_type.id),
            ("index".to_string(), int_type.id),
        ],
        int_type.id,
        "文字列の指定位置にある文字のUnicodeコードポイントを返します。",
    ));
    
    // from_char_code - 文字コードから文字列を作成
    registry.register_function(StdlibFunction::new(
        "from_char_code",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("code".to_string(), int_type.id)],
        string_type.id,
        "Unicodeコードポイントから文字列を作成します。",
    ));
    
    // is_digit - 文字列がすべて数字かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_digit",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        bool_type.id,
        "文字列がすべて数字かどうかを返します。",
    ));
    
    // is_alpha - 文字列がすべて英字かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_alpha",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        bool_type.id,
        "文字列がすべて英字かどうかを返します。",
    ));
    
    // is_alnum - 文字列がすべて英数字かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_alnum",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        bool_type.id,
        "文字列がすべて英数字かどうかを返します。",
    ));
    
    // is_whitespace - 文字列がすべて空白かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_whitespace",
        StdlibModule::String,
        StdlibFunctionType::Pure,
        vec![("str".to_string(), string_type.id)],
        bool_type.id,
        "文字列がすべて空白かどうかを返します。",
    ));
    
    Ok(())
}

/// 標準ライブラリ文字列関数の実装
pub fn execute_function(function_name: &str, args: &[String]) -> Result<String> {
    match function_name {
        "concat" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "concat関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let mut result = args[0].clone();
            result.push_str(&args[1]);
            Ok(result)
        },
        "substr" => {
            if args.len() != 3 {
                return Err(EidosError::Runtime(format!(
                    "substr関数は3つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let start: usize = args[1].parse().map_err(|_| {
                EidosError::Runtime("startパラメータを整数に変換できません".to_string())
            })?;
            let length: usize = args[2].parse().map_err(|_| {
                EidosError::Runtime("lengthパラメータを整数に変換できません".to_string())
            })?;
            
            let chars: Vec<char> = str.chars().collect();
            if start >= chars.len() {
                return Ok(String::new());
            }
            
            let end = std::cmp::min(start + length, chars.len());
            let result: String = chars[start..end].iter().collect();
            Ok(result)
        },
        "to_upper" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "to_upper関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].to_uppercase())
        },
        "to_lower" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "to_lower関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].to_lowercase())
        },
        "trim" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "trim関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].trim().to_string())
        },
        "trim_start" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "trim_start関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].trim_start().to_string())
        },
        "trim_end" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "trim_end関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].trim_end().to_string())
        },
        "replace" => {
            if args.len() != 3 {
                return Err(EidosError::Runtime(format!(
                    "replace関数は3つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].replace(&args[1], &args[2]))
        },
        "replace_first" => {
            if args.len() != 3 {
                return Err(EidosError::Runtime(format!(
                    "replace_first関数は3つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let from = &args[1];
            let to = &args[2];
            
            if let Some(pos) = str.find(from) {
                let mut result = str.clone();
                result.replace_range(pos..pos + from.len(), to);
                Ok(result)
            } else {
                Ok(str.clone())
            }
        },
        "repeat" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "repeat関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let count: usize = args[1].parse().map_err(|_| {
                EidosError::Runtime("countパラメータを整数に変換できません".to_string())
            })?;
            
            Ok(str.repeat(count))
        },
        "from_char_code" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "from_char_code関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let code: u32 = args[0].parse().map_err(|_| {
                EidosError::Runtime("codeパラメータを整数に変換できません".to_string())
            })?;
            
            if let Some(ch) = std::char::from_u32(code) {
                Ok(ch.to_string())
            } else {
                Err(EidosError::Runtime(format!("無効なユニコードコードポイント: {}", code)))
            }
        },
        // 他の文字列関数はランタイムシステムで提供
        _ => Err(EidosError::Runtime(format!("未実装の文字列関数: {}", function_name))),
    }
}

/// 文字列関数の実装（ブール値を返すもの）
pub fn execute_bool_function(function_name: &str, args: &[String]) -> Result<bool> {
    match function_name {
        "is_empty" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_empty関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].is_empty())
        },
        "contains" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "contains関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].contains(&args[1]))
        },
        "starts_with" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "starts_with関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].starts_with(&args[1]))
        },
        "ends_with" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "ends_with関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].ends_with(&args[1]))
        },
        "is_digit" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_digit関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].chars().all(|c| c.is_digit(10)))
        },
        "is_alpha" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_alpha関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].chars().all(|c| c.is_alphabetic()))
        },
        "is_alnum" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_alnum関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].chars().all(|c| c.is_alphanumeric()))
        },
        "is_whitespace" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_whitespace関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].chars().all(|c| c.is_whitespace()))
        },
        // 他の文字列関数はランタイムシステムで提供
        _ => Err(EidosError::Runtime(format!("未実装の文字列関数: {}", function_name))),
    }
}

/// 文字列関数の実装（整数値を返すもの）
pub fn execute_int_function(function_name: &str, args: &[String]) -> Result<i64> {
    match function_name {
        "length" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "length関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].chars().count() as i64)
        },
        "index_of" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "index_of関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let substr = &args[1];
            
            // Unicodeのコードポイントを考慮した位置を計算
            if let Some(byte_pos) = str.find(substr) {
                let prefix = &str[..byte_pos];
                Ok(prefix.chars().count() as i64)
            } else {
                Ok(-1)
            }
        },
        "last_index_of" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "last_index_of関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let substr = &args[1];
            
            // Unicodeのコードポイントを考慮した位置を計算
            if let Some(byte_pos) = str.rfind(substr) {
                let prefix = &str[..byte_pos];
                Ok(prefix.chars().count() as i64)
            } else {
                Ok(-1)
            }
        },
        "char_at" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "char_at関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let index: usize = args[1].parse().map_err(|_| {
                EidosError::Runtime("indexパラメータを整数に変換できません".to_string())
            })?;
            
            let chars: Vec<char> = str.chars().collect();
            if index < chars.len() {
                Ok(chars[index] as i64)
            } else {
                Err(EidosError::Runtime(format!("インデックスが範囲外です: {}", index)))
            }
        },
        // 他の文字列関数はランタイムシステムで提供
        _ => Err(EidosError::Runtime(format!("未実装の文字列関数: {}", function_name))),
    }
}

/// 文字列関数の実装（文字列配列を返すもの）
pub fn execute_string_array_function(function_name: &str, args: &[String]) -> Result<Vec<String>> {
    match function_name {
        "split" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "split関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let delimiter = &args[1];
            
            let parts: Vec<String> = str.split(delimiter).map(|s| s.to_string()).collect();
            Ok(parts)
        },
        // 他の文字列関数はランタイムシステムで提供
        _ => Err(EidosError::Runtime(format!("未実装の文字列関数: {}", function_name))),
    }
}

/// 文字列関数の実装（文字を返すもの）
pub fn execute_char_function(function_name: &str, args: &[String]) -> Result<char> {
    match function_name {
        "at" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "at関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let str = &args[0];
            let index: usize = args[1].parse().map_err(|_| {
                EidosError::Runtime("indexパラメータを整数に変換できません".to_string())
            })?;
            
            let chars: Vec<char> = str.chars().collect();
            if index < chars.len() {
                Ok(chars[index])
            } else {
                Err(EidosError::Runtime(format!("インデックスが範囲外です: {}", index)))
            }
        },
        // 他の文字列関数はランタイムシステムで提供
        _ => Err(EidosError::Runtime(format!("未実装の文字列関数: {}", function_name))),
    }
} 