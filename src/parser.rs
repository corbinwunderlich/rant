use std::{
    sync::atomic::{AtomicU8, Ordering},
    vec::IntoIter,
};

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

trait Parsable: Sized {
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error>;
}

type Ident = String;
type Type = String;

static PARENTHESIS_DEPTH: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, PartialEq)]
pub struct TypedIdent(pub Ident, pub Type);

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(u64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Term(Term),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
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
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error> {
        let ident = match tokens.next() {
            Some(Token::Ident(ident)) => ident,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "identifier",
                    got: format!("{token:?}"),
                });
            }
            _ => return Err(Error::Parse),
        };

        if let token = tokens.next()
            && token != Some(Token::Colon)
        {
            return Err(Error::UnexpectedToken {
                expected: "`:`",
                got: format!("{token:?}"),
            });
        }

        let ty = match tokens.next() {
            Some(Token::Ident(ident)) => ident,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "identifier",
                    got: format!("{token:?}"),
                });
            }
            _ => return Err(Error::Parse),
        };

        Ok(TypedIdent(ident, ty))
    }
}

impl Parsable for Expression {
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error> {
        let term = Self::Term(Term::parse(tokens)?);

        tokens.reset_peek();

        match tokens.peek().ok_or(Error::UnexpectedEof)? {
            Token::Semicolon | Token::Comma => {
                tokens.next();

                Ok(term)
            }
            Token::Add => {
                tokens.next();

                Ok(Self::Add(Box::new(term), Box::new(Self::parse(tokens)?)))
            }
            Token::Subtract => {
                tokens.next();

                Ok(Self::Subtract(
                    Box::new(term),
                    Box::new(Self::parse(tokens)?),
                ))
            }
            Token::Multiply => {
                tokens.next();

                Ok(Self::Multiply(
                    Box::new(term),
                    Box::new(Self::parse(tokens)?),
                ))
            }
            Token::Divide => {
                tokens.next();

                Ok(Self::Divide(Box::new(term), Box::new(Self::parse(tokens)?)))
            }
            _ => Ok(term),
        }
    }
}

impl Parsable for FunctionCall {
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error> {
        let ident = match tokens.next() {
            Some(Token::Ident(ident)) => ident,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "identifier",
                    got: format!("{token:?}"),
                });
            }
            _ => return Err(Error::Parse),
        };

        if let token = tokens.next()
            && token != Some(Token::LeftParen)
        {
            return Err(Error::UnexpectedToken {
                expected: "`(`",
                got: format!("{token:?}"),
            });
        }

        let mut params: Vec<Expression> = Vec::new();

        let starting_parenthesis_depth = PARENTHESIS_DEPTH.load(Ordering::Relaxed);

        while let Some(token) = tokens.peek()
            && *token != Token::Semicolon
            && !(*token == Token::RightParen
                && PARENTHESIS_DEPTH.load(Ordering::Relaxed) > starting_parenthesis_depth)
        {
            params.push(Expression::parse(tokens)?);

            if *tokens.peek().ok_or(Error::UnexpectedEof)? == Token::RightParen {
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
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error> {
        tokens.reset_peek();

        let next_token = tokens.peek().ok_or(Error::UnexpectedEof)?;

        match next_token {
            Token::Number(number) => {
                let number = *number;

                tokens.next();

                Ok(Self::Literal(Literal::Number(number)))
            }
            Token::String(string) => {
                let string = string.clone();

                tokens.next();

                Ok(Self::Literal(Literal::String(string)))
            }
            Token::Ident(ident) => {
                let ident = ident.clone();

                if let Some(Token::LeftParen) = tokens.peek() {
                    PARENTHESIS_DEPTH.fetch_add(1, Ordering::Relaxed);

                    return Ok(Self::FunctionCall(FunctionCall::parse(tokens)?));
                }

                tokens.next();

                Ok(Self::Ident(ident))
            }
            token => Err(Error::UnexpectedToken {
                expected: "number, string, or identifier",
                got: format!("{token:?}"),
            }),
        }
    }
}

impl Parsable for Declaration {
    fn parse(tokens: &mut MultiPeek<IntoIter<Token>>) -> Result<Self, Error> {
        if let token = tokens.next()
            && token != Some(Token::Fn)
        {
            return Err(Error::UnexpectedToken {
                expected: "`fn`",
                got: format!("{token:?}"),
            });
        }

        let ident = match tokens.next() {
            Some(Token::Ident(ident)) => ident,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "identifier",
                    got: format!("{token:?}"),
                });
            }
            _ => return Err(Error::Parse),
        };

        if let token = tokens.next()
            && token != Some(Token::LeftParen)
        {
            return Err(Error::UnexpectedToken {
                expected: "`(`",
                got: format!("{token:?}"),
            });
        }

        let mut params: Vec<TypedIdent> = Vec::new();

        while let Some(token) = tokens.peek() {
            if *token == Token::RightParen {
                tokens.next();

                break;
            }

            params.push(TypedIdent::parse(tokens)?);

            match tokens.next().ok_or(Error::UnexpectedEof)? {
                Token::Comma => continue,
                Token::RightParen => break,
                token => {
                    return Err(Error::UnexpectedToken {
                        expected: "`,` or `)`",
                        got: format!("{token:?}"),
                    });
                }
            }
        }

        if let token = tokens.next()
            && token != Some(Token::Arrow)
        {
            return Err(Error::UnexpectedToken {
                expected: "`->`",
                got: format!("{token:?}"),
            });
        }

        let return_type = match tokens.next() {
            Some(Token::Ident(ident)) => ident,
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "identifier",
                    got: format!("{token:?}"),
                });
            }
            _ => return Err(Error::Parse),
        };

        if let token = tokens.next()
            && token != Some(Token::Equals)
        {
            return Err(Error::UnexpectedToken {
                expected: "`=`",
                got: format!("{token:?}"),
            });
        }

        Ok(Self::Function {
            ident,
            params: params.into_boxed_slice(),
            return_type,
            body: Expression::parse(tokens)?,
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Node, Error> {
    let mut tokens = tokens.into_iter().multipeek();

    let mut declarations: Vec<Declaration> = Vec::new();

    while tokens.peek().is_some() {
        tokens.reset_peek();

        declarations.push(Declaration::parse(&mut tokens)?);
    }

    Ok(Node::Program(
        declarations
            .into_iter()
            .map(Node::Declaration)
            .collect::<Vec<Node>>()
            .into_boxed_slice(),
    ))
}
