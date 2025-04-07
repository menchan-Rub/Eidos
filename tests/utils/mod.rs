//! テスト用ユーティリティ関数

use std::path::PathBuf;
use std::process::Command;
use std::fs;

/// 一時テストファイルを作成する
pub fn create_test_file(content: &str, filename: &str) -> std::io::Result<PathBuf> {
    let path = PathBuf::from(filename);
    fs::write(&path, content)?;
    Ok(path)
}

/// テストファイルを削除する（エラーは無視）
pub fn cleanup_test_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

/// Eidosコンパイラのチェックコマンドを実行
pub fn run_eidos_check(file_path: &PathBuf) -> Result<String, String> {
    let output = Command::new("target/debug/eidos")
        .args(["check", &file_path.to_string_lossy()])
        .output();
        
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Eidosコンパイラのビルドコマンドを実行
pub fn run_eidos_build(file_path: &PathBuf, output_path: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new("target/debug/eidos");
    cmd.arg("build").arg(&file_path.to_string_lossy());
    
    if let Some(output) = output_path {
        cmd.args(["-o", output]);
    }
    
    let output = cmd.output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Eidosコンパイラの実行コマンドを実行
pub fn run_eidos_run(file_path: &PathBuf, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("target/debug/eidos");
    cmd.arg("run").arg(&file_path.to_string_lossy());
    
    for arg in args {
        cmd.arg(arg);
    }
    
    let output = cmd.output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// ASTノードの表示用フォーマッタ
pub trait AstFormatter {
    fn format(&self) -> String;
}

/// テスト用のサンプルEidosコード
pub mod samples {
    /// 単純な関数定義
    pub const SIMPLE_FUNCTION: &str = r#"
        fn add(a: Int, b: Int): Int {
            return a + b;
        }
        
        fn main(): Int {
            return add(1, 2);
        }
    "#;
    
    /// 条件分岐を含むコード
    pub const IF_CONDITION: &str = r#"
        fn max(a: Int, b: Int): Int {
            if a > b {
                return a;
            } else {
                return b;
            }
        }
        
        fn main(): Int {
            return max(10, 20);
        }
    "#;
    
    /// 再帰関数
    pub const RECURSIVE_FUNCTION: &str = r#"
        fn factorial(n: Int): Int {
            if n <= 1 {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
        
        fn main(): Int {
            return factorial(5);
        }
    "#;
    
    /// グローバル変数を含むコード
    pub const GLOBAL_VARIABLE: &str = r#"
        let PI: Float = 3.14159;
        
        fn calculate_circle_area(radius: Float): Float {
            return PI * radius * radius;
        }
        
        fn main(): Float {
            return calculate_circle_area(2.0);
        }
    "#;
} 