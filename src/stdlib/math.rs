use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};

/// 数学モジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 型の登録
    let int_type = Type::int();
    let float_type = Type::float();
    let bool_type = Type::bool();
    
    // 定数の登録 (定数もシンボルとして関数で表現)
    
    // PI - 円周率
    registry.register_function(StdlibFunction::new(
        "PI",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![],
        float_type.id,
        "円周率の値（3.14159265358979323846...）",
    ));
    
    // E - 自然対数の底
    registry.register_function(StdlibFunction::new(
        "E",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![],
        float_type.id,
        "自然対数の底（2.7182818284590452354...）",
    ));
    
    // 基本的な数学関数
    
    // abs - 絶対値 (整数)
    registry.register_function(StdlibFunction::new(
        "abs_i",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), int_type.id)],
        int_type.id,
        "整数の絶対値を返します。",
    ));
    
    // abs - 絶対値 (浮動小数点数)
    registry.register_function(StdlibFunction::new(
        "abs_f",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "浮動小数点数の絶対値を返します。",
    ));
    
    // sqrt - 平方根
    registry.register_function(StdlibFunction::new(
        "sqrt",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の平方根を返します。",
    ));
    
    // cbrt - 立方根
    registry.register_function(StdlibFunction::new(
        "cbrt",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の立方根を返します。",
    ));
    
    // pow - べき乗
    registry.register_function(StdlibFunction::new(
        "pow",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("base".to_string(), float_type.id),
            ("exponent".to_string(), float_type.id),
        ],
        float_type.id,
        "baseのexponent乗を返します。",
    ));
    
    // exp - 指数関数
    registry.register_function(StdlibFunction::new(
        "exp",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "e（自然対数の底）のvalue乗を返します。",
    ));
    
    // log - 自然対数
    registry.register_function(StdlibFunction::new(
        "log",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の自然対数を返します。",
    ));
    
    // log10 - 常用対数
    registry.register_function(StdlibFunction::new(
        "log10",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の常用対数（底が10の対数）を返します。",
    ));
    
    // log2 - 二進対数
    registry.register_function(StdlibFunction::new(
        "log2",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の二進対数（底が2の対数）を返します。",
    ));
    
    // 三角関数
    
    // sin - 正弦
    registry.register_function(StdlibFunction::new(
        "sin",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("radians".to_string(), float_type.id)],
        float_type.id,
        "角度（ラジアン）の正弦を返します。",
    ));
    
    // cos - 余弦
    registry.register_function(StdlibFunction::new(
        "cos",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("radians".to_string(), float_type.id)],
        float_type.id,
        "角度（ラジアン）の余弦を返します。",
    ));
    
    // tan - 正接
    registry.register_function(StdlibFunction::new(
        "tan",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("radians".to_string(), float_type.id)],
        float_type.id,
        "角度（ラジアン）の正接を返します。",
    ));
    
    // asin - 逆正弦
    registry.register_function(StdlibFunction::new(
        "asin",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の逆正弦（アークサイン）をラジアンで返します。",
    ));
    
    // acos - 逆余弦
    registry.register_function(StdlibFunction::new(
        "acos",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の逆余弦（アークコサイン）をラジアンで返します。",
    ));
    
    // atan - 逆正接
    registry.register_function(StdlibFunction::new(
        "atan",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の逆正接（アークタンジェント）をラジアンで返します。",
    ));
    
    // atan2 - 2引数の逆正接
    registry.register_function(StdlibFunction::new(
        "atan2",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("y".to_string(), float_type.id),
            ("x".to_string(), float_type.id),
        ],
        float_type.id,
        "y/xの逆正接をラジアンで返します。x、yの符号を使用して象限を決定します。",
    ));
    
    // 双曲線関数
    
    // sinh - 双曲線正弦
    registry.register_function(StdlibFunction::new(
        "sinh",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の双曲線正弦を返します。",
    ));
    
    // cosh - 双曲線余弦
    registry.register_function(StdlibFunction::new(
        "cosh",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の双曲線余弦を返します。",
    ));
    
    // tanh - 双曲線正接
    registry.register_function(StdlibFunction::new(
        "tanh",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の双曲線正接を返します。",
    ));
    
    // 角度変換
    
    // degrees - ラジアンから度に変換
    registry.register_function(StdlibFunction::new(
        "degrees",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("radians".to_string(), float_type.id)],
        float_type.id,
        "ラジアンから度に変換します。",
    ));
    
    // radians - 度からラジアンに変換
    registry.register_function(StdlibFunction::new(
        "radians",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("degrees".to_string(), float_type.id)],
        float_type.id,
        "度からラジアンに変換します。",
    ));
    
    // 丸め関数
    
    // ceil - 切り上げ
    registry.register_function(StdlibFunction::new(
        "ceil",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値以上の最小の整数を返します（切り上げ）。",
    ));
    
    // floor - 切り捨て
    registry.register_function(StdlibFunction::new(
        "floor",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値以下の最大の整数を返します（切り捨て）。",
    ));
    
    // round - 四捨五入
    registry.register_function(StdlibFunction::new(
        "round",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値を最も近い整数に丸めます（四捨五入）。",
    ));
    
    // trunc - 小数部分を切り捨て
    registry.register_function(StdlibFunction::new(
        "trunc",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "数値の小数部分を切り捨てて整数部分を返します。",
    ));
    
    // その他の関数
    
    // min - 最小値 (整数)
    registry.register_function(StdlibFunction::new(
        "min_i",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("a".to_string(), int_type.id),
            ("b".to_string(), int_type.id),
        ],
        int_type.id,
        "2つの整数のうち小さい方を返します。",
    ));
    
    // min - 最小値 (浮動小数点数)
    registry.register_function(StdlibFunction::new(
        "min_f",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("a".to_string(), float_type.id),
            ("b".to_string(), float_type.id),
        ],
        float_type.id,
        "2つの浮動小数点数のうち小さい方を返します。",
    ));
    
    // max - 最大値 (整数)
    registry.register_function(StdlibFunction::new(
        "max_i",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("a".to_string(), int_type.id),
            ("b".to_string(), int_type.id),
        ],
        int_type.id,
        "2つの整数のうち大きい方を返します。",
    ));
    
    // max - 最大値 (浮動小数点数)
    registry.register_function(StdlibFunction::new(
        "max_f",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("a".to_string(), float_type.id),
            ("b".to_string(), float_type.id),
        ],
        float_type.id,
        "2つの浮動小数点数のうち大きい方を返します。",
    ));
    
    // clamp - 値を範囲内に制限 (整数)
    registry.register_function(StdlibFunction::new(
        "clamp_i",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("value".to_string(), int_type.id),
            ("min".to_string(), int_type.id),
            ("max".to_string(), int_type.id),
        ],
        int_type.id,
        "整数値をmin以上max以下の範囲に制限します。",
    ));
    
    // clamp - 値を範囲内に制限 (浮動小数点数)
    registry.register_function(StdlibFunction::new(
        "clamp_f",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![
            ("value".to_string(), float_type.id),
            ("min".to_string(), float_type.id),
            ("max".to_string(), float_type.id),
        ],
        float_type.id,
        "浮動小数点数値をmin以上max以下の範囲に制限します。",
    ));
    
    // is_nan - 数値がNaNかどうかを判定
    registry.register_function(StdlibFunction::new(
        "is_nan",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        bool_type.id,
        "数値がNaN（非数）かどうかを判定します。",
    ));
    
    // is_infinite - 数値が無限大かどうかを判定
    registry.register_function(StdlibFunction::new(
        "is_infinite",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        bool_type.id,
        "数値が無限大かどうかを判定します。",
    ));
    
    // is_finite - 数値が有限かどうかを判定
    registry.register_function(StdlibFunction::new(
        "is_finite",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        bool_type.id,
        "数値が有限かどうかを判定します。",
    ));
    
    // sign - 符号関数 (整数)
    registry.register_function(StdlibFunction::new(
        "sign_i",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), int_type.id)],
        int_type.id,
        "整数の符号を返します（負なら-1、0なら0、正なら1）。",
    ));
    
    // sign - 符号関数 (浮動小数点数)
    registry.register_function(StdlibFunction::new(
        "sign_f",
        StdlibModule::Math,
        StdlibFunctionType::Pure,
        vec![("value".to_string(), float_type.id)],
        float_type.id,
        "浮動小数点数の符号を返します（負なら-1.0、0なら0.0、正なら1.0）。",
    ));
    
    Ok(())
}

/// 標準ライブラリ数学関数の実装
pub fn execute_function(function_name: &str, args: &[f64]) -> Result<f64> {
    match function_name {
        // 定数
        "PI" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "PI定数は引数が不要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(std::f64::consts::PI)
        },
        "E" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "E定数は引数が不要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(std::f64::consts::E)
        },
        
        // 基本的な数学関数
        "abs_f" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "abs_f関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].abs())
        },
        "sqrt" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sqrt関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].sqrt())
        },
        "cbrt" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "cbrt関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].cbrt())
        },
        "pow" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "pow関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].powf(args[1]))
        },
        "exp" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "exp関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].exp())
        },
        "log" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "log関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].ln())
        },
        "log10" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "log10関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].log10())
        },
        "log2" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "log2関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].log2())
        },
        
        // 三角関数
        "sin" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sin関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].sin())
        },
        "cos" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "cos関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].cos())
        },
        "tan" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "tan関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].tan())
        },
        "asin" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "asin関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].asin())
        },
        "acos" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "acos関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].acos())
        },
        "atan" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "atan関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].atan())
        },
        "atan2" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "atan2関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].atan2(args[1]))
        },
        
        // 双曲線関数
        "sinh" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sinh関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].sinh())
        },
        "cosh" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "cosh関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].cosh())
        },
        "tanh" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "tanh関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].tanh())
        },
        
        // 角度変換
        "degrees" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "degrees関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0] * 180.0 / std::f64::consts::PI)
        },
        "radians" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "radians関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0] * std::f64::consts::PI / 180.0)
        },
        
        // 丸め関数
        "ceil" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "ceil関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].ceil())
        },
        "floor" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "floor関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].floor())
        },
        "round" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "round関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].round())
        },
        "trunc" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "trunc関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].trunc())
        },
        
        // その他の関数
        "min_f" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "min_f関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].min(args[1]))
        },
        "max_f" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "max_f関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].max(args[1]))
        },
        "clamp_f" => {
            if args.len() != 3 {
                return Err(EidosError::Runtime(format!(
                    "clamp_f関数は3つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].max(args[1]).min(args[2]))
        },
        "is_nan" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_nan関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(if args[0].is_nan() { 1.0 } else { 0.0 })
        },
        "is_infinite" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_infinite関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(if args[0].is_infinite() { 1.0 } else { 0.0 })
        },
        "is_finite" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "is_finite関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(if args[0].is_finite() { 1.0 } else { 0.0 })
        },
        "sign_f" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sign_f関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let val = args[0];
            if val > 0.0 {
                Ok(1.0)
            } else if val < 0.0 {
                Ok(-1.0)
            } else {
                Ok(0.0)
            }
        },
        
        // 未実装の関数
        _ => Err(EidosError::Runtime(format!("未実装の数学関数: {}", function_name))),
    }
}

/// 整数演算のための関数の実装
pub fn execute_int_function(function_name: &str, args: &[i64]) -> Result<i64> {
    match function_name {
        // 整数演算
        "abs_i" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "abs_i関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].abs())
        },
        "min_i" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "min_i関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].min(args[1]))
        },
        "max_i" => {
            if args.len() != 2 {
                return Err(EidosError::Runtime(format!(
                    "max_i関数は2つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].max(args[1]))
        },
        "clamp_i" => {
            if args.len() != 3 {
                return Err(EidosError::Runtime(format!(
                    "clamp_i関数は3つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            Ok(args[0].max(args[1]).min(args[2]))
        },
        "sign_i" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sign_i関数は1つの引数が必要ですが、{}個の引数が提供されました", args.len()
                )));
            }
            let val = args[0];
            if val > 0 {
                Ok(1)
            } else if val < 0 {
                Ok(-1)
            } else {
                Ok(0)
            }
        },
        
        // 未実装の関数
        _ => Err(EidosError::Runtime(format!("未実装の整数数学関数: {}", function_name))),
    }
} 