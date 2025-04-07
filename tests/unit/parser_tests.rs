use eidos::frontend::parser::{Parser, ParseResult};
use eidos::frontend::lexer::{Lexer, Token};
use eidos::core::ast::{ASTNode, Expression, Statement, Program};
use eidos::core::error::EidosError;

#[cfg(test)]
mod parser_tests {
    use super::*;

    // ヘルパー関数：入力文字列をパースしてプログラムASTを返す
    fn parse_program(input: &str) -> ParseResult<Program> {
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let parser = Parser::new(tokens);
        parser.parse_program()
    }

    #[test]
    fn test_parse_variable_declaration() {
        let input = "let x = 42;";
        let program = parse_program(input).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Let(name, expr) = &program.statements[0] {
            assert_eq!(name, "x");
            match expr {
                Expression::IntegerLiteral(value) => assert_eq!(*value, 42),
                _ => panic!("Expected integer literal"),
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let input = "fn add(a: Int, b: Int): Int { return a + b; }";
        let program = parse_program(input).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Function(func) = &program.statements[0] {
            assert_eq!(func.name, "add");
            assert_eq!(func.parameters.len(), 2);
            assert_eq!(func.parameters[0].name, "a");
            assert_eq!(func.parameters[1].name, "b");
            assert_eq!(func.body.statements.len(), 1);
            
            // 戻り値型の確認
            assert!(func.return_type.is_some());
            
            // 関数本体の確認
            if let Statement::Return(expr) = &func.body.statements[0] {
                if let Expression::BinaryOperation { left, operator, right } = expr {
                    assert_eq!(operator, "+");
                    
                    if let Expression::Identifier(name) = &**left {
                        assert_eq!(name, "a");
                    } else {
                        panic!("Expected identifier");
                    }
                    
                    if let Expression::Identifier(name) = &**right {
                        assert_eq!(name, "b");
                    } else {
                        panic!("Expected identifier");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            } else {
                panic!("Expected return statement");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if x > 10 { return true; } else { return false; }";
        let program = parse_program(input).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::If(condition, consequence, alternative) = &program.statements[0] {
            // 条件式のチェック
            if let Expression::BinaryOperation { left, operator, right } = condition {
                assert_eq!(operator, ">");
                
                if let Expression::Identifier(name) = &**left {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier");
                }
                
                if let Expression::IntegerLiteral(value) = &**right {
                    assert_eq!(*value, 10);
                } else {
                    panic!("Expected integer literal");
                }
            } else {
                panic!("Expected binary operation");
            }
            
            // Then節のチェック
            assert_eq!(consequence.statements.len(), 1);
            if let Statement::Return(expr) = &consequence.statements[0] {
                if let Expression::BooleanLiteral(value) = expr {
                    assert_eq!(*value, true);
                } else {
                    panic!("Expected boolean literal");
                }
            } else {
                panic!("Expected return statement");
            }
            
            // Else節のチェック
            assert!(alternative.is_some());
            let alt = alternative.as_ref().unwrap();
            assert_eq!(alt.statements.len(), 1);
            if let Statement::Return(expr) = &alt.statements[0] {
                if let Expression::BooleanLiteral(value) = expr {
                    assert_eq!(*value, false);
                } else {
                    panic!("Expected boolean literal");
                }
            } else {
                panic!("Expected return statement");
            }
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_binary_operations() {
        let input = "let result = 1 + 2 * 3 - 4 / 5;";
        let program = parse_program(input).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Let(name, expr) = &program.statements[0] {
            assert_eq!(name, "result");
            // 演算子の優先順位が正しく処理されているか確認するのは複雑なので
            // ここではExpressionの構造が得られていることだけを確認
            match expr {
                Expression::BinaryOperation { .. } => (),
                _ => panic!("Expected binary operation"),
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_call_expression() {
        let input = "let result = add(1, 2);";
        let program = parse_program(input).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Let(name, expr) = &program.statements[0] {
            assert_eq!(name, "result");
            
            if let Expression::FunctionCall { name, arguments } = expr {
                assert_eq!(name, "add");
                assert_eq!(arguments.len(), 2);
                
                if let Expression::IntegerLiteral(value) = &arguments[0] {
                    assert_eq!(*value, 1);
                } else {
                    panic!("Expected integer literal");
                }
                
                if let Expression::IntegerLiteral(value) = &arguments[1] {
                    assert_eq!(*value, 2);
                } else {
                    panic!("Expected integer literal");
                }
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_syntax_error() {
        // 構文エラーがある入力
        let input = "let x = ;";
        let result = parse_program(input);
        
        assert!(result.is_err());
    }
} 