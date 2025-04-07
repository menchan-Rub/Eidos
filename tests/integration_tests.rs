mod unit;

#[cfg(test)]
mod integration_tests {
    use std::path::Path;
    use std::process::Command;
    
    // シンプルなプログラムをファイルに書き込む
    fn create_test_file(content: &str, path: &str) -> std::io::Result<()> {
        std::fs::write(path, content)
    }
    
    // 型チェックのテスト
    #[test]
    fn test_type_check_command() {
        let test_file = "test_type_check.eid";
        let content = r#"
            fn add(x: Int, y: Int): Int {
                return x + y;
            }
            
            fn main(): Int {
                return add(1, 2);
            }
        "#;
        
        // テストファイル作成
        create_test_file(content, test_file).expect("Failed to create test file");
        
        // 型チェックコマンド実行
        let output = Command::new("target/debug/eidos")
            .args(["check", test_file])
            .output();
            
        // 後片付け
        std::fs::remove_file(test_file).ok();
        
        // 型チェックコマンドが存在しない場合はスキップ
        if output.is_err() {
            println!("Type check command not available, skipping test");
            return;
        }
        
        let output = output.unwrap();
        assert!(output.status.success(), "Type check failed: {:?}", 
            String::from_utf8_lossy(&output.stderr));
    }
    
    // ビルドコマンドのテスト
    #[test]
    fn test_build_command() {
        let test_file = "test_build.eid";
        let content = r#"
            fn main(): Int {
                return 42;
            }
        "#;
        
        // テストファイル作成
        create_test_file(content, test_file).expect("Failed to create test file");
        
        // ビルドコマンド実行
        let output = Command::new("target/debug/eidos")
            .args(["build", test_file])
            .output();
            
        // 後片付け
        std::fs::remove_file(test_file).ok();
        if Path::new("test_build").exists() {
            std::fs::remove_file("test_build").ok();
        }
        
        // ビルドコマンドが存在しない場合はスキップ
        if output.is_err() {
            println!("Build command not available, skipping test");
            return;
        }
        
        let output = output.unwrap();
        assert!(output.status.success(), "Build failed: {:?}", 
            String::from_utf8_lossy(&output.stderr));
    }
    
    // DSL構文テスト
    #[test]
    fn test_dsl_syntax() {
        let test_file = "test_dsl.eid";
        let content = r#"
            syntax arithmetic {
              rule expr = term "+" term;
              rule term = number;
            }
            
            semantics expr => |a, b| a + b;
            
            fn main(): Int {
                return 42;
            }
        "#;
        
        // テストファイル作成
        create_test_file(content, test_file).expect("Failed to create test file");
        
        // 型チェックコマンド実行
        let output = Command::new("target/debug/eidos")
            .args(["check", test_file])
            .output();
            
        // 後片付け
        std::fs::remove_file(test_file).ok();
        
        // コマンドが存在しない場合はスキップ
        if output.is_err() {
            println!("Check command not available, skipping test");
            return;
        }
        
        let output = output.unwrap();
        assert!(output.status.success(), "DSL syntax check failed: {:?}", 
            String::from_utf8_lossy(&output.stderr));
    }
} 