use std::fmt::{Display, Formatter, Result as FmtResult, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Reserved words
    Program,
    Begin,
    End,
    Var,
    Integer,
    If,
    Then,
    Else,
    Do,
    While,

    // Identifier & Number
    Id(String),
    Number(u32),

    // Operators & Delimiters
    Plus,
    Minus,
    LeftParen,
    RightParen,
    Equal,
    GreaterThan,
    LessThan,
    Semicolon,
    Comma,
    Colon,
    Assign,
}

impl Token {
    pub const fn serial_id(&self) -> u16 {
        match self {
            Token::Program => 1,
            Token::Begin => 2,
            Token::End => 3,
            Token::Var => 4,
            Token::Integer => 5,
            Token::If => 6,
            Token::Then => 7,
            Token::Else => 8,
            Token::Do => 9,
            Token::While => 10,
            Token::Id(_) => 11,
            Token::Number(_) => 12,
            Token::Plus => 13,
            Token::Minus => 14,
            Token::LeftParen => 15,
            Token::RightParen => 16,
            Token::Equal => 17,
            Token::GreaterThan => 18,
            Token::LessThan => 19,
            Token::Semicolon => 20,
            Token::Comma => 21,
            Token::Colon => 22,
            Token::Assign => 23,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({},", self.serial_id())?;
        match self {
            Token::Id(id) => f.write_str(id),
            Token::Number(num) => num.fmt(f),
            _ => f.write_char('_'),
        }?;
        f.write_char(')')
    }
}
