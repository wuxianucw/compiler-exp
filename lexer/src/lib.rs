pub mod token;

use std::{io::{BufRead, Bytes, Error as IOError}, iter::FusedIterator};

use thiserror::Error;

pub use token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid byte `{0}` at line {}, column {}", .1.line, .1.column)]
    InvalidByte(u8, Position),

    #[error("IO error occurred")]
    IOError(#[from] IOError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Lexer<R> {
    bytes: Bytes<R>,
    current: Option<u8>,
    position: Position,
}

impl Position {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

impl<R: BufRead> Lexer<R> {
    pub fn new(reader: R) -> Result<Self> {
        let mut bytes = reader.bytes();
        let current = bytes.next().transpose().map_err(Error::IOError)?;
        Ok(Self {
            bytes,
            current,
            position: Position::new(1, 0),
        })
    }

    fn advance(&mut self) -> Result<()> {
        if matches!(self.current, Some(b'\n')) {
            self.position.line += 1;
            self.position.column = 0;
        } else {
            self.position.column += 1;
        }
        self.current = self.bytes.next().transpose().map_err(Error::IOError)?;
        Ok(())
    }

    fn skip_whitespace(&mut self) -> Result<()> {
        while self.current.is_some_and(|x| x.is_ascii_whitespace()) {
            self.advance()?;
        }
        Ok(())
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace()?;
        if self.current.is_none() {
            return Ok(None);
        }

        let current = self.current.unwrap();
        let current_pos = self.position;
        self.advance()?;
        match current {
            b'+' => Ok(Some(Token::Plus)),
            b'-' => Ok(Some(Token::Minus)),
            b'(' => Ok(Some(Token::LeftParen)),
            b')' => Ok(Some(Token::RightParen)),
            b'=' => Ok(Some(Token::Equal)),
            b'>' => Ok(Some(Token::GreaterThan)),
            b'<' => Ok(Some(Token::LessThan)),
            b';' => Ok(Some(Token::Semicolon)),
            b',' => Ok(Some(Token::Comma)),
            b':' => {
                match self.current {
                    Some(b'=') => {
                        self.advance()?;
                        Ok(Some(Token::Assign))
                    }
                    _ => Ok(Some(Token::Colon)),
                }
            }
            x @ b'0'..=b'9' => {
                let mut num = (x - b'0') as u32;
                while self.current.is_some_and(|x| x.is_ascii_digit()) {
                    let digit = self.current.unwrap() - b'0';
                    num = num * 10 + digit as u32;
                    self.advance()?; // FIXME: retry capability
                }
                Ok(Some(Token::Number(num)))
            }
            x @ (b'A'..=b'Z' | b'a'..=b'z' | b'_') => {
                let mut buf = vec![x];
                while self.current.is_some_and(|x| x.is_ascii_alphanumeric() || x == b'_') {
                    buf.push(self.current.unwrap());
                    self.advance()?;
                }
                let word = String::from_utf8(buf).unwrap();
                Ok(Some(match &word[..] {
                    "program" | "Program" | "PROGRAM" => Token::Program,
                    "begin" | "Begin" | "BEGIN" => Token::Begin,
                    "end" | "End" | "END" => Token::End,
                    "var" | "Var" | "VAR" => Token::Var,
                    "integer" | "Integer" | "INTEGER" => Token::Integer,
                    "if" | "If" | "IF" => Token::If,
                    "then" | "Then" | "THEN" => Token::Then,
                    "else" | "Else" | "ELSE" => Token::Else,
                    "do" | "Do" | "DO" => Token::Do,
                    "while" | "While" | "WHILE" => Token::While,
                    _ => Token::Id(word)
                }))
            }
            x => Err(Error::InvalidByte(x, current_pos))
        }
    }
}

impl<R: BufRead> Iterator for Lexer<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().transpose()
    }
}

impl<R: BufRead> FusedIterator for Lexer<R> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize() {
        let input = "  \n\tprogram;:=_aaa1_b0 vAr:VAR 1234567890 Var6".as_bytes();
        let mut lexer = Lexer::new(input).unwrap();

        assert_eq!(lexer.next_token().unwrap(), Some(Token::Program));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Semicolon));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Assign));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Id("_aaa1_b0".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Id("vAr".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Colon));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Var));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Number(1234567890)));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Id("Var6".into())));
        assert_eq!(lexer.next_token().unwrap(), None);
    }

    #[test]
    fn non_ascii_is_invalid() {
        let input = "你好，世界。".as_bytes();
        let invalid_byte = input[0];

        let mut lexer = Lexer::new(input).unwrap();
        let result = lexer.next().unwrap();

        match result.unwrap_err() {
            Error::InvalidByte(byte, position) => {
                assert_eq!(byte, invalid_byte);
                assert_eq!(position, Position::new(1, 0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn invalid_byte_position() {
        let input = "program\nprog\x01ram".as_bytes();
        let invalid_byte = b'\x01';
        let invalid_pos = Position::new(2, 4);

        let mut lexer = Lexer::new(input).unwrap();
        let result = lexer.nth(2).unwrap();

        match result.unwrap_err() {
            Error::InvalidByte(byte, position) => {
                assert_eq!(byte, invalid_byte);
                assert_eq!(position, invalid_pos);
            }
            _ => panic!(),
        }
    }
}
