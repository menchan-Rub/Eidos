use eidos::frontend::lexer::{Lexer, Token, TokenType};

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "let x = 42;";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].literal, "x");
        assert_eq!(tokens[2].token_type, TokenType::Assign);
        assert_eq!(tokens[3].token_type, TokenType::Integer);
        assert_eq!(tokens[3].literal, "42");
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
    }

    #[test]
    fn test_lexer_keywords() {
        let input = "fn if else return while for";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Function);
        assert_eq!(tokens[1].token_type, TokenType::If);
        assert_eq!(tokens[2].token_type, TokenType::Else);
        assert_eq!(tokens[3].token_type, TokenType::Return);
        assert_eq!(tokens[4].token_type, TokenType::While);
        assert_eq!(tokens[5].token_type, TokenType::For);
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+ - * / < > == != && ||";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Asterisk);
        assert_eq!(tokens[3].token_type, TokenType::Slash);
        assert_eq!(tokens[4].token_type, TokenType::LessThan);
        assert_eq!(tokens[5].token_type, TokenType::GreaterThan);
        assert_eq!(tokens[6].token_type, TokenType::Equal);
        assert_eq!(tokens[7].token_type, TokenType::NotEqual);
        assert_eq!(tokens[8].token_type, TokenType::LogicalAnd);
        assert_eq!(tokens[9].token_type, TokenType::LogicalOr);
    }

    #[test]
    fn test_lexer_complex_code() {
        let input = r#"
            fn factorial(n: Int): Int {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
        "#;
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // 必要なトークンが含まれているか検証
        let keywords = tokens.iter().filter(|t| matches!(t.token_type, 
            TokenType::Function | TokenType::If | TokenType::Else | TokenType::Return
        )).count();
        
        assert_eq!(keywords, 4); // fn, if, else, return x2
        
        // 識別子の検証
        let identifiers: Vec<&str> = tokens.iter()
            .filter(|t| t.token_type == TokenType::Identifier)
            .map(|t| t.literal.as_str())
            .collect();
        
        assert!(identifiers.contains(&"factorial"));
        assert!(identifiers.contains(&"n"));
        assert!(identifiers.contains(&"Int"));
    }

    #[test]
    fn test_lexer_string_literals() {
        let input = r#"let message = "Hello, World!";"#;
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].literal, "message");
        assert_eq!(tokens[2].token_type, TokenType::Assign);
        assert_eq!(tokens[3].token_type, TokenType::String);
        assert_eq!(tokens[3].literal, "Hello, World!");
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
    }

    #[test]
    fn test_lexer_numeric_literals() {
        let input = "42 3.14 0.123 5000";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].literal, "42");
        assert_eq!(tokens[1].token_type, TokenType::Float);
        assert_eq!(tokens[1].literal, "3.14");
        assert_eq!(tokens[2].token_type, TokenType::Float);
        assert_eq!(tokens[2].literal, "0.123");
        assert_eq!(tokens[3].token_type, TokenType::Integer);
        assert_eq!(tokens[3].literal, "5000");
    }

    #[test]
    fn test_lexer_delimiters() {
        let input = "{ } ( ) [ ] , ;";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[1].token_type, TokenType::RightBrace);
        assert_eq!(tokens[2].token_type, TokenType::LeftParen);
        assert_eq!(tokens[3].token_type, TokenType::RightParen);
        assert_eq!(tokens[4].token_type, TokenType::LeftBracket);
        assert_eq!(tokens[5].token_type, TokenType::RightBracket);
        assert_eq!(tokens[6].token_type, TokenType::Comma);
        assert_eq!(tokens[7].token_type, TokenType::Semicolon);
    }

    #[test]
    fn test_lexer_comments() {
        let input = r#"
            // これは行コメントです
            let x = 10; // 行末コメント
            /* これは
               複数行コメント
               です */
            let y = 20;
        "#;
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // コメントはトークン化されないことを確認
        let identifiers: Vec<&str> = tokens.iter()
            .filter(|t| t.token_type == TokenType::Identifier)
            .map(|t| t.literal.as_str())
            .collect();
        
        assert_eq!(identifiers, vec!["x", "y"]);
        
        // 数値リテラルの確認
        let numbers: Vec<&str> = tokens.iter()
            .filter(|t| t.token_type == TokenType::Integer)
            .map(|t| t.literal.as_str())
            .collect();
        
        assert_eq!(numbers, vec!["10", "20"]);
    }

    #[test]
    fn test_lexer_advanced_operators() {
        let input = "= += -= *= /= %= << >> & | ^ ~ <= >= !";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Assign);
        assert_eq!(tokens[1].token_type, TokenType::PlusAssign);
        assert_eq!(tokens[2].token_type, TokenType::MinusAssign);
        assert_eq!(tokens[3].token_type, TokenType::MultiplyAssign);
        assert_eq!(tokens[4].token_type, TokenType::DivideAssign);
        assert_eq!(tokens[5].token_type, TokenType::ModuloAssign);
        assert_eq!(tokens[6].token_type, TokenType::ShiftLeft);
        assert_eq!(tokens[7].token_type, TokenType::ShiftRight);
        assert_eq!(tokens[8].token_type, TokenType::BitwiseAnd);
        assert_eq!(tokens[9].token_type, TokenType::BitwiseOr);
        assert_eq!(tokens[10].token_type, TokenType::BitwiseXor);
        assert_eq!(tokens[11].token_type, TokenType::BitwiseNot);
        assert_eq!(tokens[12].token_type, TokenType::LessThanEqual);
        assert_eq!(tokens[13].token_type, TokenType::GreaterThanEqual);
        assert_eq!(tokens[14].token_type, TokenType::Bang);
    }

    #[test]
    fn test_lexer_error_handling() {
        // 無効な文字を含む入力
        let input = "let x = @;";
        let lexer = Lexer::new(input);
        let result = lexer.tokenize();
        
        assert!(result.is_err());
        
        // 閉じられていない文字列
        let input = r#"let message = "Hello, World!;"#;
        let lexer = Lexer::new(input);
        let result = lexer.tokenize();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_lexer_source_location() {
        let input = "let x = 42;\nlet y = 10;";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // 1行目のトークンの位置を確認
        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);
        
        // 2行目のトークンの位置を確認
        assert_eq!(tokens[5].location.line, 2);
        assert_eq!(tokens[5].location.column, 1);
    }

    #[test]
    fn test_lexer_type_annotations() {
        let input = "fn add(a: Int, b: Float): Float { return a as Float + b; }";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // 型アノテーションの確認
        let type_identifiers: Vec<&str> = tokens.iter()
            .filter(|t| t.token_type == TokenType::Identifier && (t.literal == "Int" || t.literal == "Float"))
            .map(|t| t.literal.as_str())
            .collect();
        
        assert_eq!(type_identifiers, vec!["Int", "Float", "Float", "Float"]);
        
        // as キーワードの確認
        let as_keywords = tokens.iter()
            .filter(|t| t.token_type == TokenType::As)
            .count();
        
        assert_eq!(as_keywords, 1);
    }
}