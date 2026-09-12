use std::vec::IntoIter;

use itertools::{Itertools, MultiPeek};

use crate::lexer::Token;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("expected {expected}, got `{got}`")]
    UnexpectedToken { expected: &'static str, got: String },
    #[error("failed to parse")]
    Parse,
    #[error("unexpected end of file")]
    UnexpectedEof,
}

type TokenStream<'a> = &'a mut MultiPeek<IntoIter<Token>>;

type Ident = Box<str>;
type Type = Box<str>;

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub ident: Ident,
    pub params: Box<[Expression]>,
}

#[derive(Debug, PartialEq)]
pub enum Factor {
    Ident(Ident),
    Number(u64),
    String(Box<str>),
    FunctionCall(FunctionCall),
    Group(Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Term {
    Factor(Factor),
    Mult(Box<Term>, Box<Term>),
    Div(Box<Term>, Box<Term>),
}

#[derive(Debug, PartialEq)]
pub struct LetBindings {
    pub declarations: Box<[(Ident, Type)]>,
    pub definitions: Box<[(Ident, Expression)]>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Term(Term),
    Let(LetBindings, Box<Expression>),
    Add(Term, Box<Expression>),
    Sub(Term, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub struct FunctionDecl {
    pub ident: Ident,
    pub param_types: Box<[Type]>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDef {
    pub ident: Ident,
    pub param_idents: Box<[Ident]>,
    pub body: Expression,
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    FunctionDecl(FunctionDecl),
    FunctionDef(FunctionDef),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub nodes: Box<[Declaration]>,
}

macro_rules! expect_token {
    ($tokens:ident, $token:path) => {
        match $tokens.next() {
            Some($token) => {}
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: $token.description(),
                    got: format!("{token:?}"),
                })
            }
            None => return Err(Error::UnexpectedEof),
        }
    };
}

macro_rules! accept_token {
    ($tokens:ident, $($token:tt)+) => {
        match $tokens.next() {
            Some($($token)+(token)) => token,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: crate::lexer::token_description!($($token)+),
                    got: format!("{token:?}"),
                })
            }
            None => return Err(Error::UnexpectedEof),
        }
    };
}

fn factor(tokens: TokenStream) -> Result<Factor, Error> {
    match tokens.peek().ok_or(Error::UnexpectedEof)? {
        Token::ValueIdent(_) => {
            if let Some(Token::LeftParen) = tokens.peek() {
                let call = function_call(tokens)?;

                return Ok(Factor::FunctionCall(call));
            }

            let ident = accept_token!(tokens, Token::ValueIdent);

            Ok(Factor::Ident(ident))
        }
        Token::Number(_) => {
            let number = accept_token!(tokens, Token::Number);

            Ok(Factor::Number(number))
        }
        Token::String(_) => {
            let string = accept_token!(tokens, Token::String);

            Ok(Factor::String(string))
        }
        Token::LeftParen => {
            expect_token!(tokens, Token::LeftParen);

            let expression = Box::new(expression(tokens)?);

            expect_token!(tokens, Token::RightParen);

            Ok(Factor::Group(expression))
        }
        _ => Err(Error::UnexpectedToken {
            expected: "literal, identifier, or `(`",
            got: "none".to_string(),
        }),
    }
}

fn term(tokens: TokenStream) -> Result<Term, Error> {
    let lhs = Term::Factor(factor(tokens)?);

    if !matches!(
        tokens.peek().ok_or(Error::UnexpectedEof)?,
        Token::Asterisk | Token::Slash
    ) {
        tokens.reset_peek();

        return Ok(lhs);
    }

    let op = tokens.next().ok_or(Error::UnexpectedEof)?;

    let rhs = term(tokens)?;

    match op {
        Token::Asterisk => Ok(Term::Mult(Box::new(lhs), Box::new(rhs))),
        Token::Slash => Ok(Term::Div(Box::new(lhs), Box::new(rhs))),
        token => Err(Error::UnexpectedToken {
            expected: "`*` or `/`",
            got: format!("{token:?}"),
        }),
    }
}

fn let_in_declaration(tokens: TokenStream) -> Result<(Ident, Type), Error> {
    expect_token!(tokens, Token::Type);

    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::Equals);

    let type_ident = accept_token!(tokens, Token::TypeIdent);

    expect_token!(tokens, Token::Semicolon);

    Ok((ident, type_ident))
}

fn let_in_definition(tokens: TokenStream) -> Result<(Ident, Expression), Error> {
    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::Equals);

    let body = expression(tokens)?;

    expect_token!(tokens, Token::Semicolon);

    Ok((ident, body))
}

fn let_in_bindings(tokens: TokenStream) -> Result<LetBindings, Error> {
    expect_token!(tokens, Token::Let);

    let mut declarations: Vec<(Ident, Type)> = Vec::new();
    let mut definitions: Vec<(Ident, Expression)> = Vec::new();

    while !matches!(tokens.peek(), Some(Token::In)) {
        tokens.reset_peek();

        match tokens.peek() {
            Some(Token::Type) => {
                declarations.push(let_in_declaration(tokens)?);
            }
            Some(Token::ValueIdent(_)) => {
                definitions.push(let_in_definition(tokens)?);
            }
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: format!(
                        "{} or {}",
                        Token::Type.description(),
                        crate::lexer::token_description!(Token::ValueIdent)
                    )
                    .leak(),
                    got: format!("{token:?}"),
                });
            }
            None => return Err(Error::UnexpectedEof),
        }
    }

    expect_token!(tokens, Token::In);

    Ok(LetBindings {
        declarations: declarations.into_boxed_slice(),
        definitions: definitions.into_boxed_slice(),
    })
}

fn expression(tokens: TokenStream) -> Result<Expression, Error> {
    let let_decl = match tokens.peek() {
        Some(Token::Let) => Some(let_in_bindings(tokens)?),
        Some(_) => {
            tokens.reset_peek();

            None
        }
        None => return Err(Error::UnexpectedEof),
    };

    let lhs = term(tokens)?;

    if !matches!(
        tokens.peek().ok_or(Error::UnexpectedEof)?,
        Token::Plus | Token::Minus
    ) {
        tokens.reset_peek();

        let lhs = Expression::Term(lhs);

        if let Some(let_decl) = let_decl {
            return Ok(Expression::Let(let_decl, Box::new(lhs)));
        }

        return Ok(lhs);
    }

    let op = tokens.next().ok_or(Error::UnexpectedEof)?;

    let rhs = Box::new(expression(tokens)?);

    let expression = match op {
        Token::Plus => Expression::Add(lhs, rhs),
        Token::Minus => Expression::Sub(lhs, rhs),
        token => {
            return Err(Error::UnexpectedToken {
                expected: "`+` or `-`",
                got: format!("{token:?}"),
            });
        }
    };

    if let Some(let_decl) = let_decl {
        return Ok(Expression::Let(let_decl, Box::new(expression)));
    }

    Ok(expression)
}

fn function_call(tokens: TokenStream) -> Result<FunctionCall, Error> {
    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::LeftParen);

    let mut params: Vec<Expression> = Vec::new();

    while *tokens.peek().ok_or(Error::UnexpectedEof)? != Token::RightParen {
        tokens.reset_peek();

        let param = expression(tokens)?;

        params.push(param);

        if *tokens.peek().ok_or(Error::UnexpectedEof)? == Token::RightParen {
            break;
        }

        expect_token!(tokens, Token::Comma);
    }

    expect_token!(tokens, Token::RightParen);

    Ok(FunctionCall {
        ident,
        params: params.into_boxed_slice(),
    })
}

fn function_declaration(tokens: TokenStream) -> Result<FunctionDecl, Error> {
    expect_token!(tokens, Token::Type);
    expect_token!(tokens, Token::Fn);

    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::Equals);
    expect_token!(tokens, Token::LeftParen);

    let mut params: Vec<Type> = Vec::new();

    while let Some(Token::TypeIdent(_)) = tokens.peek() {
        let ident = accept_token!(tokens, Token::TypeIdent);

        params.push(ident);

        if *tokens.peek().ok_or(Error::UnexpectedEof)? == Token::RightParen {
            break;
        }

        expect_token!(tokens, Token::Comma);
    }

    expect_token!(tokens, Token::RightParen);
    expect_token!(tokens, Token::Arrow);

    let return_type = accept_token!(tokens, Token::TypeIdent);

    expect_token!(tokens, Token::Semicolon);

    Ok(FunctionDecl {
        ident,
        param_types: params.into_boxed_slice(),
        return_type,
    })
}

fn function_definition(tokens: TokenStream) -> Result<FunctionDef, Error> {
    expect_token!(tokens, Token::Fn);

    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::Equals);
    expect_token!(tokens, Token::LeftParen);

    let mut params: Vec<Ident> = Vec::new();

    while let Some(Token::ValueIdent(_)) = tokens.peek() {
        let ident = accept_token!(tokens, Token::ValueIdent);

        params.push(ident);

        if *tokens.peek().ok_or(Error::UnexpectedEof)? == Token::RightParen {
            break;
        }

        expect_token!(tokens, Token::Comma);
    }

    expect_token!(tokens, Token::RightParen);
    expect_token!(tokens, Token::Colon);

    let body = expression(tokens)?;

    expect_token!(tokens, Token::Semicolon);

    Ok(FunctionDef {
        ident,
        param_idents: params.into_boxed_slice(),
        body,
    })
}

fn declaration(tokens: TokenStream) -> Result<Declaration, Error> {
    match tokens.peek() {
        Some(Token::Type) => Ok(Declaration::FunctionDecl(function_declaration(tokens)?)),
        Some(Token::Fn) => Ok(Declaration::FunctionDef(function_definition(tokens)?)),
        Some(token) => Err(Error::UnexpectedToken {
            expected: format!(
                "{} or {}",
                Token::Type.description(),
                Token::Fn.description()
            )
            .leak(),
            got: format!("{token:?}"),
        }),
        None => Err(Error::UnexpectedEof),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, Error> {
    let mut tokens = tokens.into_iter().multipeek();

    let mut declarations: Vec<Declaration> = Vec::new();

    while tokens.peek().is_some() {
        tokens.reset_peek();

        declarations.push(declaration(&mut tokens)?);
    }

    Ok(Program {
        nodes: declarations.into_boxed_slice(),
    })
}
