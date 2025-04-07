use eidos::frontend::type_checker::{TypeChecker, TypeCheckResult};
use eidos::core::ast::{ASTNode, Expression, Statement};
use eidos::core::types::{Type, TypeEnvironment};
use eidos::core::error::EidosError;

#[cfg(test)]
mod type_checker_tests {
    use super::*;

    // ヘルパー関数：AST作成用
    fn create_int_literal(value: i64) -> Expression {
        Expression::IntegerLiteral(value)
    }

    fn create_string_literal(value: String) -> Expression {
        Expression::StringLiteral(value)
    }

    fn create_identifier(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    fn create_binary_op(left: Expression, operator: &str, right: Expression) -> Expression {
        Expression::BinaryOperation {
            left: Box::new(left),
            operator: operator.to_string(),
            right: Box::new(right),
        }
    }

    #[test]
    fn test_integer_type_check() {
        let mut type_env = TypeEnvironment::new();
        let type_checker = TypeChecker::new();
        
        let int_expr = create_int_literal(42);
        let result = type_checker.check_expression(&int_expr, &mut type_env);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Int);
    }

    #[test]
    fn test_string_type_check() {
        let mut type_env = TypeEnvironment::new();
        let type_checker = TypeChecker::new();
        
        let string_expr = create_string_literal("hello".to_string());
        let result = type_checker.check_expression(&string_expr, &mut type_env);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::String);
    }

    #[test]
    fn test_binary_op_type_check() {
        let mut type_env = TypeEnvironment::new();
        let type_checker = TypeChecker::new();
        
        // 整数加算
        let int_add = create_binary_op(
            create_int_literal(1),
            "+",
            create_int_literal(2)
        );
        
        let result = type_checker.check_expression(&int_add, &mut type_env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Int);
        
        // 型の不一致（整数 + 文字列）
        let invalid_add = create_binary_op(
            create_int_literal(1),
            "+",
            create_string_literal("hello".to_string())
        );
        
        let result = type_checker.check_expression(&invalid_add, &mut type_env);
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_type_check() {
        let mut type_env = TypeEnvironment::new();
        let type_checker = TypeChecker::new();
        
        // 変数を環境に追加
        type_env.define("x".to_string(), Type::Int);
        
        // 存在する変数
        let x_expr = create_identifier("x");
        let result = type_checker.check_expression(&x_expr, &mut type_env);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Int);
        
        // 存在しない変数
        let y_expr = create_identifier("y");
        let result = type_checker.check_expression(&y_expr, &mut type_env);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_function_call_type_check() {
        let mut type_env = TypeEnvironment::new();
        let type_checker = TypeChecker::new();
        
        // 関数シグネチャを環境に追加
        type_env.define_function(
            "add".to_string(),
            vec![Type::Int, Type::Int],
            Type::Int
        );
        
        // 正しい型の引数
        let valid_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                create_int_literal(1),
                create_int_literal(2),
            ],
        };
        
        let result = type_checker.check_expression(&valid_call, &mut type_env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Int);
        
        // 引数の数が違う
        let invalid_arg_count = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                create_int_literal(1),
            ],
        };
        
        let result = type_checker.check_expression(&invalid_arg_count, &mut type_env);
        assert!(result.is_err());
        
        // 引数の型が違う
        let invalid_arg_type = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                create_int_literal(1),
                create_string_literal("hello".to_string()),
            ],
        };
        
        let result = type_checker.check_expression(&invalid_arg_type, &mut type_env);
        assert!(result.is_err());
    }
} 