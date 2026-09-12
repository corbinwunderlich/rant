use std::num::ParseIntError;

use logos::Logos;

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
    #[regex("[a-z_][a-z0-9_']*", |lex| lex.slice().parse::<String>().ok().map(Into::into))]
    ValueIdent(Box<str>),
    #[regex("[A-Z][a-zA-Z0-9]*", |lex| lex.slice().parse::<String>().ok().map(Into::into))]
    TypeIdent(Box<str>),
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(u64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();

        s[1..s.len() - 1].parse::<String>().ok().map(Into::into)
    })]
    String(Box<str>),
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("type")]
    Type,
    #[token("fn")]
    Fn,
    #[token(":")]
    Colon,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("->")]
    Arrow,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[regex("#.*", logos::skip, priority = 255, allow_greedy = true)]
    Comment,
}

macro_rules! token_description {
    (Token::ValueIdent) => {
        "identifier"
    };
    (Token::TypeIdent) => {
        "type"
    };
    (Token::Number) => {
        "number"
    };
    (Token::String) => {
        "string"
    };
    (Token::Let) => {
        "`let`"
    };
    (Token::In) => {
        "`in`"
    };
    (Token::Type) => {
        "`type`"
    };
    (Token::Fn) => {
        "`fn`"
    };
    (Token::Colon) => {
        "`:`"
    };
    (Token::Plus) => {
        "`+`"
    };
    (Token::Minus) => {
        "`-`"
    };
    (Token::Asterisk) => {
        "`*`"
    };
    (Token::Slash) => {
        "`/`"
    };
    (Token::Equals) => {
        "`=`"
    };
    (Token::Comma) => {
        "`,`"
    };
    (Token::Semicolon) => {
        "`;`"
    };
    (Token::Arrow) => {
        "`->`"
    };
    (Token::LeftParen) => {
        "`(`"
    };
    (Token::RightParen) => {
        "`)`"
    };
    (Token::Comment) => {
        "`#`"
    };
}

impl Token {
    #[must_use]
    pub fn description(&self) -> &'static str {
        match *self {
            Self::ValueIdent(_) => token_description!(Token::ValueIdent),
            Self::TypeIdent(_) => token_description!(Token::TypeIdent),
            Self::Number(_) => token_description!(Token::Number),
            Self::String(_) => token_description!(Token::String),
            Self::Let => token_description!(Token::Let),
            Self::In => token_description!(Token::In),
            Self::Fn => token_description!(Token::Fn),
            Self::Type => token_description!(Token::Type),
            Self::Colon => token_description!(Token::Colon),
            Self::Plus => token_description!(Token::Plus),
            Self::Minus => token_description!(Token::Minus),
            Self::Asterisk => token_description!(Token::Asterisk),
            Self::Slash => token_description!(Token::Slash),
            Self::Equals => token_description!(Token::Equals),
            Self::Comma => token_description!(Token::Comma),
            Self::Semicolon => token_description!(Token::Semicolon),
            Self::Arrow => token_description!(Token::Arrow),
            Self::LeftParen => token_description!(Token::LeftParen),
            Self::RightParen => token_description!(Token::RightParen),
            Self::Comment => token_description!(Token::Comment),
        }
    }
}

pub(crate) use token_description;

pub fn tokenize(source: &str) -> Result<Vec<Token>, Error> {
    let lex = Token::lexer(source).collect::<Result<Vec<Token>, Error>>()?;

    Ok(lex)
}
