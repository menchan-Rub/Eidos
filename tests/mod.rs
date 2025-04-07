//! Eidosコンパイラのテストスイート

// ユニットテスト
mod unit;

// 統合テスト
mod integration_tests;

// テストユーティリティ
mod utils;

// PRプロセスでの検証テスト
#[cfg(test)]
mod verification {
    use super::utils::*;
    use std::path::PathBuf;
    
    /// サンプルコードがすべて正常にコンパイルできることを確認
    #[test]
    fn test_samples_compile() {
        // サンプルごとにコンパイルテスト
        let samples = [
            ("simple_function.eid", samples::SIMPLE_FUNCTION),
            ("if_condition.eid", samples::IF_CONDITION),
            ("recursive_function.eid", samples::RECURSIVE_FUNCTION),
            ("global_variable.eid", samples::GLOBAL_VARIABLE),
        ];
        
        for (filename, content) in samples.iter() {
            let temp_file = create_test_file(content, filename).unwrap();
            
            let result = run_eidos_check(&temp_file);
            cleanup_test_file(&temp_file);
            
            if let Err(err) = result {
                panic!("Sample {} failed to compile: {}", filename, err);
            }
        }
    }
    
    /// コンパイルに失敗すべきコードが適切にエラーを返すことを確認
    #[test]
    fn test_compiler_errors() {
        // 型エラー
        let type_error = r#"
            fn add(a: Int, b: Int): Int {
                return a + "string"; // 型エラー: 文字列と整数の加算
            }
        "#;
        
        let temp_file = create_test_file(type_error, "type_error.eid").unwrap();
        let result = run_eidos_check(&temp_file);
        cleanup_test_file(&temp_file);
        
        assert!(result.is_err(), "型エラーを検出できませんでした");
        
        // 構文エラー
        let syntax_error = r#"
            fn missing_brace() {
                return 42;
            // 閉じ括弧がない
        "#;
        
        let temp_file = create_test_file(syntax_error, "syntax_error.eid").unwrap();
        let result = run_eidos_check(&temp_file);
        cleanup_test_file(&temp_file);
        
        assert!(result.is_err(), "構文エラーを検出できませんでした");
        
        // 未定義変数エラー
        let undefined_var = r#"
            fn main(): Int {
                return undefined_variable; // 未定義変数
            }
        "#;
        
        let temp_file = create_test_file(undefined_var, "undefined_var.eid").unwrap();
        let result = run_eidos_check(&temp_file);
        cleanup_test_file(&temp_file);
        
        assert!(result.is_err(), "未定義変数エラーを検出できませんでした");
    }
    
    /// コンパイラが正しいコードの場合に成功することを確認
    #[test]
    fn test_compiler_success() {
        let valid_code = r#"
            fn factorial(n: Int): Int {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            
            fn main(): Int {
                return factorial(5); // Should return 120
            }
        "#;
        
        let temp_file = create_test_file(valid_code, "valid.eid").unwrap();
        let result = run_eidos_check(&temp_file);
        cleanup_test_file(&temp_file);
        
        assert!(result.is_ok(), "有効なコードがエラーになりました: {:?}", result.err());
    }
} 