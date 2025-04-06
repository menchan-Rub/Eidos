use std::fmt;
use std::path::PathBuf;
use std::str::Chars;

use crate::core::{EidosError, Result, SourceLocation};

/// トークンの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // リテラル
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
    Boolean(bool),
    
    // 識別子
    Identifier(String),
    
    // キーワード
    Let,
    Var,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Type,
    Struct,
    Enum,
    Import,
    Export,
    Unsafe,
    As,
    Mut,
    True,
    False,
    
    // 区切り文字
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    Semicolon,     // ;
    Colon,         // :
    Comma,         // ,
    Dot,           // .
    Arrow,         // ->
    
    // 演算子
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Ampersand,     // &
    Pipe,          // |
    Caret,         // ^
    Bang,          // !
    Equal,         // =
    EqualEqual,    // ==
    BangEqual,     // !=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    AmpersandAmpersand, // &&
    PipePipe,      // ||
    LessLess,      // <<
    GreaterGreater, // >>
    
    // DSL関連
    DSLStart(String), // `@dsl_name {`
    DSLEnd,           // `}`
    
    // その他
    Eof,
    Unknown(char),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // リテラル
            TokenKind::Integer(val) => write!(f, "{}", val),
            TokenKind::Float(val) => write!(f, "{}", val),
            TokenKind::String(val) => write!(f, "\"{}\"", val),
            TokenKind::Character(val) => write!(f, "'{}'", val),
            TokenKind::Boolean(val) => write!(f, "{}", val),
            
            // 識別子
            TokenKind::Identifier(name) => write!(f, "{}", name),
            
            // キーワード
            TokenKind::Let => write!(f, "let"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Unsafe => write!(f, "unsafe"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            
            // 区切り文字
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Arrow => write!(f, "->"),
            
            // 演算子
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::AmpersandAmpersand => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::LessLess => write!(f, "<<"),
            TokenKind::GreaterGreater => write!(f, ">>"),
            
            // DSL関連
            TokenKind::DSLStart(name) => write!(f, "@{} {{", name),
            TokenKind::DSLEnd => write!(f, "}}"),
            
            // その他
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Unknown(c) => write!(f, "Unknown({})", c),
        }
    }
}

/// トークン
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLocation,
}

impl Token {
    pub fn new(kind: TokenKind, location: SourceLocation) -> Self {
        Self { kind, location }
    }
}

/// 字句解析器
pub struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
    file_path: PathBuf,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file_path: PathBuf) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        
        Self {
            input,
            chars,
            current,
            line: 1,
            column: 1,
            file_path,
        }
    }
    
    /// 現在の位置のソース位置情報を取得
    fn current_location(&self, length: usize) -> SourceLocation {
        SourceLocation::new(
            self.file_path.clone(),
            self.line,
            self.column,
            length,
        )
    }
    
    /// 現在の文字が指定した文字と一致するかチェック
    fn match_char(&mut self, expected: char) -> bool {
        match self.current {
            Some(c) if c == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }
    
    /// 次の文字に進む
    fn advance(&mut self) {
        if let Some('\n') = self.current {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.current = self.chars.next();
    }
    
    /// 空白をスキップ
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// コメントをスキップ
    fn skip_comment(&mut self) {
        if self.current == Some('/') {
            if self.peek() == Some('/') {
                // 行コメント
                self.advance(); // 最初の '/' をスキップ
                self.advance(); // 2つ目の '/' をスキップ
                
                while let Some(c) = self.current {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if self.peek() == Some('*') {
                // ブロックコメント
                self.advance(); // '/' をスキップ
                self.advance(); // '*' をスキップ
                
                let mut nesting = 1;
                
                while nesting > 0 {
                    match self.current {
                        Some('*') if self.peek() == Some('/') => {
                            self.advance(); // '*' をスキップ
                            self.advance(); // '/' をスキップ
                            nesting -= 1;
                        }
                        Some('/') if self.peek() == Some('*') => {
                            self.advance(); // '/' をスキップ
                            self.advance(); // '*' をスキップ
                            nesting += 1;
                        }
                        Some(_) => self.advance(),
                        None => break, // 終端に達した（不正なコメント）
                    }
                }
            }
        }
    }
    
    /// 空白とコメントをスキップ
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            
            if self.current == Some('/') && (self.peek() == Some('/') || self.peek() == Some('*')) {
                self.skip_comment();
            } else {
                break;
            }
        }
    }
    
    /// 次の文字をピーク（先読み）
    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
    
    /// 数値リテラルを解析
    fn number(&mut self) -> TokenKind {
        let start_column = self.column;
        let mut value = 0;
        let mut is_float = false;
        let mut decimal_place = 0.1;
        
        // 整数部分を解析
        while let Some(c) = self.current {
            if c.is_digit(10) {
                let digit = c.to_digit(10).unwrap() as i64;
                value = value * 10 + digit;
                self.advance();
            } else if c == '.' && self.peek().map_or(false, |c| c.is_digit(10)) {
                is_float = true;
                self.advance(); // '.' をスキップ
                break;
            } else {
                break;
            }
        }
        
        // 小数部分を解析
        if is_float {
            let mut float_value = value as f64;
            
            while let Some(c) = self.current {
                if c.is_digit(10) {
                    let digit = c.to_digit(10).unwrap() as f64;
                    float_value += digit * decimal_place;
                    decimal_place *= 0.1;
                    self.advance();
                } else {
                    break;
                }
            }
            
            TokenKind::Float(float_value)
        } else {
            TokenKind::Integer(value)
        }
    }
    
    /// 識別子またはキーワードを解析
    fn identifier(&mut self) -> TokenKind {
        let start_column = self.column;
        let mut name = String::new();
        
        while let Some(c) = self.current {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // キーワードをチェック
        match name.as_str() {
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "type" => TokenKind::Type,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "unsafe" => TokenKind::Unsafe,
            "as" => TokenKind::As,
            "mut" => TokenKind::Mut,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(name),
        }
    }
    
    /// 文字列リテラルを解析
    fn string(&mut self) -> Result<TokenKind> {
        let mut value = String::new();
        
        // 開始の引用符をスキップ
        self.advance();
        
        while let Some(c) = self.current {
            if c == '"' {
                // 終了の引用符をスキップ
                self.advance();
                return Ok(TokenKind::String(value));
            } else if c == '\\' {
                // エスケープシーケンス
                self.advance();
                match self.current {
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some(c) => {
                        return Err(EidosError::Lexer {
                            message: format!("不明なエスケープシーケンス: \\{}", c),
                            file: self.file_path.clone(),
                            line: self.line,
                            column: self.column,
                        });
                    },
                    None => {
                        return Err(EidosError::Lexer {
                            message: "文字列リテラルが途中で終了しました".to_string(),
                            file: self.file_path.clone(),
                            line: self.line,
                            column: self.column,
                        });
                    },
                }
                self.advance();
            } else {
                value.push(c);
                self.advance();
            }
        }
        
        Err(EidosError::Lexer {
            message: "文字列リテラルが閉じられていません".to_string(),
            file: self.file_path.clone(),
            line: self.line,
            column: self.column,
        })
    }
    
    /// 文字リテラルを解析
    fn character(&mut self) -> Result<TokenKind> {
        // 開始のシングルクォートをスキップ
        self.advance();
        
        let c = match self.current {
            Some('\\') => {
                // エスケープシーケンス
                self.advance();
                match self.current {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some(c) => {
                        return Err(EidosError::Lexer {
                            message: format!("不明なエスケープシーケンス: \\{}", c),
                            file: self.file_path.clone(),
                            line: self.line,
                            column: self.column,
                        });
                    },
                    None => {
                        return Err(EidosError::Lexer {
                            message: "文字リテラルが途中で終了しました".to_string(),
                            file: self.file_path.clone(),
                            line: self.line,
                            column: self.column,
                        });
                    },
                }
            },
            Some(c) => c,
            None => {
                return Err(EidosError::Lexer {
                    message: "文字リテラルが空です".to_string(),
                    file: self.file_path.clone(),
                    line: self.line,
                    column: self.column,
                });
            },
        };
        
        self.advance();
        
        if self.current != Some('\'') {
            return Err(EidosError::Lexer {
                message: "文字リテラルが閉じられていません".to_string(),
                file: self.file_path.clone(),
                line: self.line,
                column: self.column,
            });
        }
        
        // 終了のシングルクォートをスキップ
        self.advance();
        
        Ok(TokenKind::Character(c))
    }
    
    /// DSL開始トークンを解析（@name { 形式）
    fn dsl_start(&mut self) -> Result<TokenKind> {
        // '@' をスキップ
        self.advance();
        
        let mut name = String::new();
        
        // DSL名を読み取り
        while let Some(c) = self.current {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            return Err(EidosError::Lexer {
                message: "DSL名が指定されていません".to_string(),
                file: self.file_path.clone(),
                line: self.line,
                column: self.column,
            });
        }
        
        // 空白をスキップ
        self.skip_whitespace();
        
        // '{' があるか確認
        if self.current != Some('{') {
            return Err(EidosError::Lexer {
                message: "DSLブロックの開始には '{' が必要です".to_string(),
                file: self.file_path.clone(),
                line: self.line,
                column: self.column,
            });
        }
        
        // '{' をスキップ
        self.advance();
        
        Ok(TokenKind::DSLStart(name))
    }
    
    /// 次のトークンを取得
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace_and_comments();
        
        if self.current.is_none() {
            return Ok(Token::new(
                TokenKind::Eof,
                self.current_location(0),
            ));
        }
        
        let start_column = self.column;
        let start_line = self.line;
        
        let kind = match self.current.unwrap() {
            // 識別子または予約語
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            
            // 数値
            c if c.is_digit(10) => self.number(),
            
            // 文字列
            '"' => self.string()?,
            
            // 文字
            '\'' => self.character()?,
            
            // DSL開始
            '@' => self.dsl_start()?,
            
            // 記号と演算子
            '(' => { self.advance(); TokenKind::LeftParen },
            ')' => { self.advance(); TokenKind::RightParen },
            '{' => { self.advance(); TokenKind::LeftBrace },
            '}' => { self.advance(); TokenKind::RightBrace },
            '[' => { self.advance(); TokenKind::LeftBracket },
            ']' => { self.advance(); TokenKind::RightBracket },
            ';' => { self.advance(); TokenKind::Semicolon },
            ':' => { self.advance(); TokenKind::Colon },
            ',' => { self.advance(); TokenKind::Comma },
            '.' => { self.advance(); TokenKind::Dot },
            
            '+' => { self.advance(); TokenKind::Plus },
            '-' => {
                self.advance();
                if self.current == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            },
            '*' => { self.advance(); TokenKind::Star },
            '/' => { self.advance(); TokenKind::Slash },
            '%' => { self.advance(); TokenKind::Percent },
            
            '&' => {
                self.advance();
                if self.current == Some('&') {
                    self.advance();
                    TokenKind::AmpersandAmpersand
                } else {
                    TokenKind::Ampersand
                }
            },
            '|' => {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    TokenKind::PipePipe
                } else {
                    TokenKind::Pipe
                }
            },
            '^' => { self.advance(); TokenKind::Caret },
            
            '!' => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            },
            '=' => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            },
            '<' => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else if self.current == Some('<') {
                    self.advance();
                    TokenKind::LessLess
                } else {
                    TokenKind::Less
                }
            },
            '>' => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else if self.current == Some('>') {
                    self.advance();
                    TokenKind::GreaterGreater
                } else {
                    TokenKind::Greater
                }
            },
            
            // 不明な文字
            c => { 
                self.advance();
                TokenKind::Unknown(c)
            },
        };
        
        // 最終的なトークンの位置と長さを計算
        let length = if self.line == start_line {
            self.column - start_column
        } else {
            // 複数行にまたがる場合（主に文字列リテラル）
            // 正確な長さは行の長さなどによって変わるため、単純化のために1とする
            1
        };
        
        let location = SourceLocation::new(
            self.file_path.clone(),
            start_line,
            start_column,
            length,
        );
        
        Ok(Token::new(kind, location))
    }
    
    /// 全てのトークンを取得
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
} 