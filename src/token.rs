use std::fmt;

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    #[default]
    Eof,
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
    String(String),
    Number(f64),
    Identifier(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String(str) => f.write_str(str),
            Object::Number(num) => f.write_str(&num.to_string()),
            Object::Identifier(ident) => f.write_str(ident),
        }
    }
}
