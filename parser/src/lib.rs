pub mod ast;

#[rustfmt::skip]
#[allow(dead_code)]
#[allow(clippy::all)]
pub mod grammar;

pub use lexer::{Position, Token};
use thiserror::Error;
// use lalrpop_util::lalrpop_mod;

// lalrpop_mod!(pub grammar);

#[derive(Debug, Error)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;
