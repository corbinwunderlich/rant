use std::num::ParseIntError;

use logos::{Lexer, Logos};

#[derive(Default, Debug, thiserror::Error, Clone, PartialEq)]
pub enum Error {
    #[error("invalid number token")]
    InvalidNumber,
    #[default]
    #[error("invalid token")]
    InvalidToken,
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error::InvalidNumber
    }
}

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"\s+", error = Error)]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().parse().ok())]
    Ident(String),
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(u64),
    #[token(":")]
    Colon,
    #[token("+")]
    Add,
    #[token("-")]
    Subtract,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("fn")]
    Fn,
    #[token("->")]
    Arrow,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[regex("#.*", |lex| lex.slice().parse().ok(), priority = 255, allow_greedy = true)]
    Comment(String),
}

pub fn tokenize(source: &str) -> Result<Lexer<'_, Token>, Error> {
    let lex = Token::lexer(source);

    Ok(lex)
}
