use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId, TypeKind, Field};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};

/// 時間モジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 基本型の登録
    let int_type = Type::int();
    let float_type = Type::float();
    let bool_type = Type::bool();
    let string_type = Type::string();
    let unit_type = Type::unit();
    
    // DateTime型の定義
    let datetime_type = Type::new(
        TypeKind::Struct {
            name: "DateTime".to_string(),
            fields: vec![
                Field {
                    name: "year".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "month".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "day".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "hour".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "minute".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "second".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "millisecond".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
                Field {
                    name: "timezone_offset".to_string(),
                    field_type: int_type.clone(),
                    is_public: true,
                },
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("time::DateTime", datetime_type.clone());
    
    // Duration型の定義
    let duration_type = Type::new(
        TypeKind::Struct {
            name: "Duration".to_string(),
            fields: vec![
                Field {
                    name: "seconds".to_string(),
                    field_type: float_type.clone(),
                    is_public: true,
                },
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("time::Duration", duration_type.clone());
    
    // 時間関数の登録
    
    // now - 現在時刻を取得
    registry.register_function(StdlibFunction::new(
        "now",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![],
        datetime_type.id,
        "現在の日時を返します。",
    ));
    
    // timestamp - UNIXタイムスタンプを取得
    registry.register_function(StdlibFunction::new(
        "timestamp",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "現在のUNIXタイムスタンプ（1970年1月1日からの秒数）を返します。",
    ));
    
    // timestamp_millis - ミリ秒単位でのUNIXタイムスタンプを取得
    registry.register_function(StdlibFunction::new(
        "timestamp_millis",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "現在のUNIXタイムスタンプをミリ秒単位で返します。",
    ));
    
    // sleep - 指定された時間だけスリープ
    registry.register_function(StdlibFunction::new(
        "sleep",
        StdlibModule::Time,
        StdlibFunctionType::Effectful,
        vec![("seconds".to_string(), float_type.id)],
        unit_type.id,
        "指定された秒数だけ現在のスレッドをスリープさせます。",
    ));
    
    // parse - 文字列から日時をパース
    registry.register_function(StdlibFunction::new(
        "parse",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("date_string".to_string(), string_type.id),
            ("format".to_string(), string_type.id),
        ],
        datetime_type.id,
        "文字列を指定されたフォーマットで解析し、DateTimeオブジェクトを返します。",
    ));
    
    // format - 日時を文字列にフォーマット
    registry.register_function(StdlibFunction::new(
        "format",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime".to_string(), datetime_type.id),
            ("format".to_string(), string_type.id),
        ],
        string_type.id,
        "DateTimeオブジェクトを指定されたフォーマットで文字列に変換します。",
    ));
    
    // add_days - 日時に日数を追加
    registry.register_function(StdlibFunction::new(
        "add_days",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime".to_string(), datetime_type.id),
            ("days".to_string(), int_type.id),
        ],
        datetime_type.id,
        "DateTimeオブジェクトに指定された日数を追加します。",
    ));
    
    // add_hours - 日時に時間を追加
    registry.register_function(StdlibFunction::new(
        "add_hours",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime".to_string(), datetime_type.id),
            ("hours".to_string(), int_type.id),
        ],
        datetime_type.id,
        "DateTimeオブジェクトに指定された時間を追加します。",
    ));
    
    // add_minutes - 日時に分を追加
    registry.register_function(StdlibFunction::new(
        "add_minutes",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime".to_string(), datetime_type.id),
            ("minutes".to_string(), int_type.id),
        ],
        datetime_type.id,
        "DateTimeオブジェクトに指定された分を追加します。",
    ));
    
    // add_seconds - 日時に秒を追加
    registry.register_function(StdlibFunction::new(
        "add_seconds",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime".to_string(), datetime_type.id),
            ("seconds".to_string(), int_type.id),
        ],
        datetime_type.id,
        "DateTimeオブジェクトに指定された秒を追加します。",
    ));
    
    // diff - 2つの日時の差を計算
    registry.register_function(StdlibFunction::new(
        "diff",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime1".to_string(), datetime_type.id),
            ("datetime2".to_string(), datetime_type.id),
        ],
        duration_type.id,
        "2つのDateTimeオブジェクト間の差をDurationとして返します。",
    ));
    
    // is_before - ある日時が別の日時より前かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_before",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime1".to_string(), datetime_type.id),
            ("datetime2".to_string(), datetime_type.id),
        ],
        bool_type.id,
        "最初のDateTimeが2番目のDateTimeより前かどうかを返します。",
    ));
    
    // is_after - ある日時が別の日時より後かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_after",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime1".to_string(), datetime_type.id),
            ("datetime2".to_string(), datetime_type.id),
        ],
        bool_type.id,
        "最初のDateTimeが2番目のDateTimeより後かどうかを返します。",
    ));
    
    // is_same - 2つの日時が同じかどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_same",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("datetime1".to_string(), datetime_type.id),
            ("datetime2".to_string(), datetime_type.id),
        ],
        bool_type.id,
        "2つのDateTimeが同じ瞬間を表しているかどうかを返します。",
    ));
    
    // create_datetime - DateTimeオブジェクトを作成
    registry.register_function(StdlibFunction::new(
        "create_datetime",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("year".to_string(), int_type.id),
            ("month".to_string(), int_type.id),
            ("day".to_string(), int_type.id),
            ("hour".to_string(), int_type.id),
            ("minute".to_string(), int_type.id),
            ("second".to_string(), int_type.id),
            ("millisecond".to_string(), int_type.id),
        ],
        datetime_type.id,
        "指定されたコンポーネントから新しいDateTimeオブジェクトを作成します。",
    ));
    
    // create_duration - Durationオブジェクトを作成
    registry.register_function(StdlibFunction::new(
        "create_duration",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("seconds".to_string(), float_type.id)],
        duration_type.id,
        "指定された秒数から新しいDurationオブジェクトを作成します。",
    ));
    
    // duration_to_seconds - Duration を秒に変換
    registry.register_function(StdlibFunction::new(
        "duration_to_seconds",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("duration".to_string(), duration_type.id)],
        float_type.id,
        "Duration を秒数に変換します。",
    ));
    
    // duration_to_milliseconds - Duration をミリ秒に変換
    registry.register_function(StdlibFunction::new(
        "duration_to_milliseconds",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("duration".to_string(), duration_type.id)],
        int_type.id,
        "Duration をミリ秒に変換します。",
    ));
    
    // datetime_to_timestamp - DateTime を UNIX タイムスタンプに変換
    registry.register_function(StdlibFunction::new(
        "datetime_to_timestamp",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime を UNIX タイムスタンプ（秒）に変換します。",
    ));
    
    // timestamp_to_datetime - UNIX タイムスタンプを DateTime に変換
    registry.register_function(StdlibFunction::new(
        "timestamp_to_datetime",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("timestamp".to_string(), int_type.id)],
        datetime_type.id,
        "UNIX タイムスタンプ（秒）を DateTime に変換します。",
    ));
    
    // get_year - DateTime から年を取得
    registry.register_function(StdlibFunction::new(
        "get_year",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から年を取得します。",
    ));
    
    // get_month - DateTime から月を取得
    registry.register_function(StdlibFunction::new(
        "get_month",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から月を取得します（1-12）。",
    ));
    
    // get_day - DateTime から日を取得
    registry.register_function(StdlibFunction::new(
        "get_day",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から日を取得します（1-31）。",
    ));
    
    // get_hour - DateTime から時間を取得
    registry.register_function(StdlibFunction::new(
        "get_hour",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から時間を取得します（0-23）。",
    ));
    
    // get_minute - DateTime から分を取得
    registry.register_function(StdlibFunction::new(
        "get_minute",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から分を取得します（0-59）。",
    ));
    
    // get_second - DateTime から秒を取得
    registry.register_function(StdlibFunction::new(
        "get_second",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から秒を取得します（0-59）。",
    ));
    
    // get_millisecond - DateTime からミリ秒を取得
    registry.register_function(StdlibFunction::new(
        "get_millisecond",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime からミリ秒を取得します（0-999）。",
    ));
    
    // get_weekday - DateTime から曜日を取得
    registry.register_function(StdlibFunction::new(
        "get_weekday",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から曜日を取得します（0 = 日曜日, 6 = 土曜日）。",
    ));
    
    // get_day_of_year - DateTime から年初からの日数を取得
    registry.register_function(StdlibFunction::new(
        "get_day_of_year",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("datetime".to_string(), datetime_type.id)],
        int_type.id,
        "DateTime から年初からの日数を取得します（1-366）。",
    ));
    
    // is_leap_year - うるう年かどうかを確認
    registry.register_function(StdlibFunction::new(
        "is_leap_year",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![("year".to_string(), int_type.id)],
        bool_type.id,
        "指定された年がうるう年かどうかを返します。",
    ));
    
    // get_days_in_month - 指定された年と月の日数を取得
    registry.register_function(StdlibFunction::new(
        "get_days_in_month",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("year".to_string(), int_type.id),
            ("month".to_string(), int_type.id),
        ],
        int_type.id,
        "指定された年と月の日数を取得します。",
    ));
    
    // get_timezone_offset - 現在のタイムゾーンオフセットを取得
    registry.register_function(StdlibFunction::new(
        "get_timezone_offset",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![],
        int_type.id,
        "現在のタイムゾーンのUTCからのオフセット（分）を取得します。",
    ));
    
    // duration_add - 2つの Duration を加算
    registry.register_function(StdlibFunction::new(
        "duration_add",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("duration1".to_string(), duration_type.id),
            ("duration2".to_string(), duration_type.id),
        ],
        duration_type.id,
        "2つの Duration を加算します。",
    ));
    
    // duration_subtract - 2つの Duration を減算
    registry.register_function(StdlibFunction::new(
        "duration_subtract",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("duration1".to_string(), duration_type.id),
            ("duration2".to_string(), duration_type.id),
        ],
        duration_type.id,
        "最初の Duration から2番目の Duration を減算します。",
    ));
    
    // duration_multiply - Duration を係数倍
    registry.register_function(StdlibFunction::new(
        "duration_multiply",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("duration".to_string(), duration_type.id),
            ("factor".to_string(), float_type.id),
        ],
        duration_type.id,
        "Duration を指定された係数で乗算します。",
    ));
    
    // duration_divide - Duration を係数で除算
    registry.register_function(StdlibFunction::new(
        "duration_divide",
        StdlibModule::Time,
        StdlibFunctionType::Pure,
        vec![
            ("duration".to_string(), duration_type.id),
            ("divisor".to_string(), float_type.id),
        ],
        duration_type.id,
        "Duration を指定された除数で除算します。",
    ));
    
    Ok(())
}

/// 時間関数の実行
pub fn execute_function(function_name: &str, args: &[String]) -> Result<String> {
    match function_name {
        "sleep" => {
            if args.len() != 1 {
                return Err(EidosError::Runtime(format!(
                    "sleep関数は1つの引数が必要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            let seconds = args[0].parse::<f64>().map_err(|_| {
                EidosError::Runtime("sleep関数の引数は数値である必要があります。".to_string())
            })?;
            
            std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
            Ok("".to_string())
        }
        "timestamp" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "timestamp関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            
            let now = std::time::SystemTime::now();
            let since_epoch = now.duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| EidosError::Runtime(format!("システム時間エラー: {}", e)))?;
            
            Ok(since_epoch.as_secs().to_string())
        }
        "timestamp_millis" => {
            if !args.is_empty() {
                return Err(EidosError::Runtime(format!(
                    "timestamp_millis関数は引数が不要ですが、{}個の引数が渡されました。",
                    args.len()
                )));
            }
            
            let now = std::time::SystemTime::now();
            let since_epoch = now.duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| EidosError::Runtime(format!("システム時間エラー: {}", e)))?;
            
            let millis = since_epoch.as_secs() * 1000 + since_epoch.subsec_millis() as u64;
            Ok(millis.to_string())
        }
        _ => Err(EidosError::Runtime(format!("時間関数 '{}' はネイティブ実装で提供されます", function_name)))
    }
} 