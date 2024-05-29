pub mod token;

use std::io::{Bytes, Read, Error as IOError};

use thiserror::Error;

pub use token::Token;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid byte `{0}`")]
    InvalidByte(u8),

    #[error("IO error occurred")]
    IOError(#[from] IOError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Lexer<R> {
    bytes: Bytes<R>,
    current: Option<u8>,
}

impl<R: Read> Lexer<R> {
    pub fn new(reader: R) -> Result<Self> {
        let mut bytes = reader.bytes();
        let current = bytes.next().transpose().map_err(Error::IOError)?;
        Ok(Self {
            bytes,
            current,
        })
    }

    fn advance(&mut self) -> Result<()> {
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
            x @ (b'A'..=b'Z' | b'a'..=b'z') => {
                let mut buf = vec![x];
                while self.current.is_some_and(|x| x.is_ascii_alphabetic()) {
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
            x => Err(Error::InvalidByte(x))
        }
    }
}

impl<R: Read> Iterator for Lexer<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().transpose()
    }
}
