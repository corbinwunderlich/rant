use std::sync::atomic::{AtomicU8, Ordering};

use itertools::{Itertools, MultiPeek};
use logos::Lexer;

use crate::lexer::Token;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse")]
    Parse,
    #[error("failed to lex")]
    Lexer(#[from] crate::lexer::Error),
    #[error("unexpected end of file")]
    UnexpectedEof,
}

trait Parsable: Sized {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error>;
}

type Ident = String;
type Type = String;

static PARENTHESIS_DEPTH: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, PartialEq)]
pub struct TypedIdent(pub Ident, pub Type);

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(u64),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Term(Term),
    Add(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall(pub Ident, pub Box<[Expression]>);

#[derive(Debug, PartialEq)]
pub enum Term {
    Literal(Literal),
    Ident(Ident),
    FunctionCall(FunctionCall),
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Function {
        ident: Ident,
        params: Box<[TypedIdent]>,
        return_type: Type,
        body: Expression,
    },
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Program(Box<[Node]>),
    Declaration(Declaration),
}

impl Parsable for TypedIdent {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error> {
        let ident = match tokens.next() {
            Some(Ok(Token::Ident(ident))) => ident,
            _ => return Err(Error::Parse),
        };

        if tokens.next() != Some(Ok(Token::Colon)) {
            return Err(Error::Parse);
        }

        let ty = match tokens.next() {
            Some(Ok(Token::Ident(ident))) => ident,
            _ => return Err(Error::Parse),
        };

        Ok(TypedIdent(ident, ty))
    }
}

impl Parsable for Expression {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error> {
        let term = Self::Term(Term::parse(tokens)?);

        tokens.reset_peek();

        match tokens.peek().ok_or(Error::Parse)?.as_ref() {
            Ok(Token::Semicolon | Token::Comma) => {
                tokens.next();

                Ok(term)
            }
            Ok(Token::Add) => {
                tokens.next();

                Ok(Self::Add(Box::new(term), Box::new(Self::parse(tokens)?)))
            }
            _ => Ok(term),
        }
    }
}

impl Parsable for FunctionCall {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error> {
        let ident = match tokens.next() {
            Some(Ok(Token::Ident(ident))) => ident,
            _ => return Err(Error::Parse),
        };

        if tokens.next() != Some(Ok(Token::LeftParen)) {
            return Err(Error::Parse);
        }

        let mut params: Vec<Expression> = Vec::new();

        let starting_parenthesis_depth = PARENTHESIS_DEPTH.load(Ordering::Relaxed);

        while let Some(Ok(token)) = tokens.peek()
            && *token != Token::Semicolon
            && !(*token == Token::RightParen
                && PARENTHESIS_DEPTH.load(Ordering::Relaxed) > starting_parenthesis_depth)
        {
            params.push(Expression::parse(tokens)?);

            if *tokens
                .peek()
                .ok_or(Error::UnexpectedEof)?
                .as_ref()
                .map_err(std::clone::Clone::clone)?
                == Token::RightParen
            {
                tokens.next();

                if PARENTHESIS_DEPTH.load(Ordering::Relaxed) > starting_parenthesis_depth {
                    break;
                }
            }

            tokens.peek();
        }

        Ok(Self(ident, params.into_boxed_slice()))
    }
}

impl Parsable for Term {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error> {
        tokens.reset_peek();

        let next_token = tokens
            .peek()
            .ok_or(Error::Parse)?
            .as_ref()
            .map_err(std::clone::Clone::clone)?;

        match next_token {
            Token::Number(number) => {
                let number = *number;

                tokens.next();

                Ok(Self::Literal(Literal::Number(number)))
            }
            Token::Ident(ident) => {
                let ident = ident.clone();

                if let Some(Ok(Token::LeftParen)) = tokens.peek() {
                    PARENTHESIS_DEPTH.fetch_add(1, Ordering::Relaxed);

                    return Ok(Self::FunctionCall(FunctionCall::parse(tokens)?));
                }

                tokens.next();

                Ok(Self::Ident(ident))
            }
            _ => Err(Error::Parse),
        }
    }
}

impl Parsable for Declaration {
    fn parse(tokens: &mut MultiPeek<Lexer<'_, Token>>) -> Result<Self, Error> {
        if tokens.next() != Some(Ok(Token::Fn)) {
            return Err(Error::Parse);
        }

        let ident = match tokens.next() {
            Some(Ok(Token::Ident(ident))) => ident,
            _ => return Err(Error::Parse),
        };

        if tokens.next() != Some(Ok(Token::LeftParen)) {
            return Err(Error::Parse);
        }

        let mut params: Vec<TypedIdent> = Vec::new();

        while let Some(Ok(token)) = tokens.peek() {
            if *token == Token::RightParen {
                tokens.next();

                break;
            }

            params.push(TypedIdent::parse(tokens)?);

            match tokens.next().ok_or(Error::UnexpectedEof)?? {
                Token::Comma => continue,
                Token::RightParen => break,
                _ => return Err(Error::Parse),
            }
        }

        if tokens.next() != Some(Ok(Token::Arrow)) {
            return Err(Error::Parse);
        }

        let return_type = match tokens.next() {
            Some(Ok(Token::Ident(ident))) => ident,
            _ => return Err(Error::Parse),
        };

        if tokens.next() != Some(Ok(Token::Equals)) {
            return Err(Error::Parse);
        }

        Ok(Self::Function {
            ident,
            params: params.into_boxed_slice(),
            return_type,
            body: Expression::parse(tokens)?,
        })
    }
}

pub fn parse(tokens: Lexer<'_, Token>) -> Result<Node, Error> {
    let mut tokens = tokens.multipeek();

    let mut declarations: Vec<Declaration> = Vec::new();

    while let Ok(declaration) = Declaration::parse(&mut tokens) {
        declarations.push(declaration);
    }

    Ok(Node::Program(
        declarations
            .into_iter()
            .map(Node::Declaration)
            .collect::<Vec<Node>>()
            .into_boxed_slice(),
    ))
}
