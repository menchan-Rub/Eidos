use std::path::PathBuf;

use crate::core::{Result, EidosError, SourceLocation};
use crate::core::ast::{ASTNode, Node, Program, Literal, UnaryOp, BinaryOp, FunctionParam};
use super::lexer::{Token, TokenKind};

/// 構文解析器
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_path: PathBuf,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file_path: PathBuf) -> Self {
        Self {
            tokens,
            current: 0,
            file_path,
        }
    }
    
    /// プログラム全体を解析
    pub fn parse(&mut self) -> Result<Program> {
        // ファイルパスを文字列に変換
        let file_path_str = self.file_path.to_string_lossy().to_string();
        
        // プログラムオブジェクトを初期化
        let mut program = Program::new(file_path_str);
        
        // EOFまで解析を続ける
        while !self.is_at_end() {
            match self.declaration() {
                Ok(node) => {
                    program.add_node(node);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(program)
    }
    
    /// 宣言を解析
    fn declaration(&mut self) -> Result<ASTNode> {
        // 現在の実装では、未実装の関数として、単に式を解析する
        self.expression()
    }
    
    /// 式を解析
    fn expression(&mut self) -> Result<ASTNode> {
        // 現在の実装では、単にリテラルを解析する
        match self.peek().kind {
            TokenKind::Integer(value) => {
                let token = self.advance();
                let location = token.location.clone();
                let literal = Literal::Int(value);
                
                Ok(ASTNode::new(Node::Literal(literal), location))
            },
            TokenKind::String(ref value) => {
                let token = self.advance();
                let location = token.location.clone();
                let literal = Literal::String(value.clone());
                
                Ok(ASTNode::new(Node::Literal(literal), location))
            },
            TokenKind::True => {
                let token = self.advance();
                let location = token.location.clone();
                let literal = Literal::Bool(true);
                
                Ok(ASTNode::new(Node::Literal(literal), location))
            },
            TokenKind::False => {
                let token = self.advance();
                let location = token.location.clone();
                let literal = Literal::Bool(false);
                
                Ok(ASTNode::new(Node::Literal(literal), location))
            },
            _ => {
                Err(EidosError::Parser {
                    message: format!("式を解析できません: {:?}", self.peek().kind),
                    file: self.file_path.clone(),
                    line: self.peek().location.line,
                    column: self.peek().location.column,
                })
            }
        }
    }
    
    /// 現在のトークンを取得して次に進む
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    /// 1つ前のトークンを取得
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    /// 現在のトークンを取得
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    /// 終端に達したかどうか
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
    
    /// 期待するトークンの場合は進む
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    /// 現在のトークンが期待するものかどうか
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }
    
    /// 期待するトークンでなければエラー
    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(EidosError::Parser {
                message: message.to_string(),
                file: self.file_path.clone(),
                line: self.peek().location.line,
                column: self.peek().location.column,
            })
        }
    }
    
    /// エラーから回復（同期化）
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            // セミコロンの後で同期
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            
            // 新しい文の始まりで同期
            match self.peek().kind {
                TokenKind::Fn | TokenKind::Let | TokenKind::Var | TokenKind::If | 
                TokenKind::While | TokenKind::Return | TokenKind::Type |
                TokenKind::Struct | TokenKind::Enum => {
                    return;
                }
                _ => {}
            }
            
            self.advance();
        }
    }
} 