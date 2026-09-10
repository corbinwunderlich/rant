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

type Ident = String;
type Type = String;

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(u64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub ident: Ident,
    pub params: Box<[Expression]>,
}

#[derive(Debug, PartialEq)]
pub enum Term {
    Literal(Literal),
    Ident(Ident),
    FunctionCall(FunctionCall),
}

#[derive(Debug, PartialEq)]
pub struct LetIn {
    pub declarations: Box<[(Ident, Type)]>,
    pub definitions: Box<[(Ident, Expression)]>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Term(Term),
    LetIn(LetIn, Box<Expression>),
    Addition(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    FunctionDeclaration {
        ident: Ident,
        param_types: Box<[Type]>,
        return_type: Type,
    },
    FunctionDefinition {
        ident: Ident,
        param_idents: Box<[Ident]>,
        body: Expression,
    },
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Program(Box<[Node]>),
    Declaration(Declaration),
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

fn term(tokens: TokenStream) -> Result<Term, Error> {
    match tokens.peek().ok_or(Error::UnexpectedEof)? {
        Token::Number(_) => {
            let number = accept_token!(tokens, Token::Number);

            Ok(Term::Literal(Literal::Number(number)))
        }
        Token::String(_) => {
            let string = accept_token!(tokens, Token::String);

            Ok(Term::Literal(Literal::String(string)))
        }
        Token::ValueIdent(_) => {
            if let Some(Token::LeftParen) = tokens.peek() {
                let call = function_call(tokens)?;

                return Ok(Term::FunctionCall(call));
            }

            let ident = accept_token!(tokens, Token::ValueIdent);

            Ok(Term::Ident(ident))
        }
        _ => Err(Error::UnexpectedToken {
            expected: "literal or identifier",
            got: "none".to_string(),
        }),
    }
}

fn let_in_declaration(tokens: TokenStream) -> Result<(Ident, Type), Error> {
    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::DoubleColon);

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

fn let_in_expression(tokens: TokenStream) -> Result<LetIn, Error> {
    expect_token!(tokens, Token::Let);

    let mut declarations: Vec<(Ident, Type)> = Vec::new();
    let mut definitions: Vec<(Ident, Expression)> = Vec::new();

    while !matches!(tokens.peek(), Some(Token::In)) {
        tokens.reset_peek();

        match tokens.peek() {
            Some(Token::ValueIdent(_)) => {}
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: crate::lexer::token_description!(Token::ValueIdent),
                    got: format!("{token:?}"),
                });
            }
            None => return Err(Error::UnexpectedEof),
        }

        match tokens.peek() {
            Some(Token::DoubleColon) => {
                declarations.push(let_in_declaration(tokens)?);
            }
            Some(Token::Equals) => {
                definitions.push(let_in_definition(tokens)?);
            }
            Some(token) => {
                return Err(Error::UnexpectedToken {
                    expected: "`::` or `=`",
                    got: format!("{token:?}"),
                });
            }
            None => return Err(Error::UnexpectedEof),
        }
    }

    expect_token!(tokens, Token::In);

    Ok(LetIn {
        declarations: declarations.into_boxed_slice(),
        definitions: definitions.into_boxed_slice(),
    })
}

fn expression(tokens: TokenStream) -> Result<Expression, Error> {
    let let_in = match tokens.peek() {
        Some(Token::Let) => Some(let_in_expression(tokens)?),
        Some(_) => {
            tokens.reset_peek();

            None
        }
        None => return Err(Error::UnexpectedEof),
    };

    let left = Expression::Term(term(tokens)?);

    macro_rules! binary_op {
        ($token:path, $expression_type:path) => {
            expect_token!(tokens, $token);

            let right = expression(tokens)?;

            let expression = $expression_type(Box::new(left), Box::new(right));

            if let Some(let_in) = let_in {
                return Ok(Expression::LetIn(let_in, Box::new(expression)));
            }

            return Ok(expression);
        };
    }

    match *tokens.peek().ok_or(Error::UnexpectedEof)? {
        Token::Plus => {
            binary_op!(Token::Plus, Expression::Addition);
        }
        Token::Minus => {
            binary_op!(Token::Minus, Expression::Subtraction);
        }
        Token::Asterisk => {
            binary_op!(Token::Asterisk, Expression::Multiplication);
        }
        Token::ForwardSlash => {
            binary_op!(Token::ForwardSlash, Expression::Division);
        }
        _ => tokens.reset_peek(),
    }

    if let Some(let_in) = let_in {
        return Ok(Expression::LetIn(let_in, Box::new(left)));
    }

    Ok(left)
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

fn function_declaration(tokens: TokenStream) -> Result<Declaration, Error> {
    expect_token!(tokens, Token::Fn);

    let ident = accept_token!(tokens, Token::ValueIdent);

    expect_token!(tokens, Token::DoubleColon);
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

    Ok(Declaration::FunctionDeclaration {
        ident,
        param_types: params.into_boxed_slice(),
        return_type,
    })
}

fn function_definition(tokens: TokenStream) -> Result<Declaration, Error> {
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

    Ok(Declaration::FunctionDefinition {
        ident,
        param_idents: params.into_boxed_slice(),
        body,
    })
}

fn declaration(tokens: TokenStream) -> Result<Declaration, Error> {
    match tokens.peek() {
        Some(Token::Fn) => {
            match tokens.peek() {
                Some(Token::ValueIdent(_)) => {}
                Some(token) => {
                    return Err(Error::UnexpectedToken {
                        expected: crate::lexer::token_description!(Token::ValueIdent),
                        got: format!("{token:?}"),
                    });
                }
                None => return Err(Error::UnexpectedEof),
            }

            match tokens.peek() {
                Some(Token::DoubleColon) => function_declaration(tokens),
                Some(Token::Equals) => function_definition(tokens),
                Some(token) => Err(Error::UnexpectedToken {
                    expected: concat!(
                        crate::lexer::token_description!(Token::DoubleColon),
                        " or ",
                        crate::lexer::token_description!(Token::Equals)
                    ),
                    got: format!("{token:?}"),
                }),
                None => Err(Error::UnexpectedEof),
            }
        }
        Some(token) => Err(Error::UnexpectedToken {
            expected: Token::Fn.description(),
            got: format!("{token:?}"),
        }),
        None => Err(Error::UnexpectedEof),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Node, Error> {
    let mut tokens = tokens.into_iter().multipeek();

    let mut declarations: Vec<Declaration> = Vec::new();

    while tokens.peek().is_some() {
        tokens.reset_peek();

        declarations.push(declaration(&mut tokens)?);
    }

    Ok(Node::Program(
        declarations
            .into_iter()
            .map(Node::Declaration)
            .collect::<Vec<Node>>()
            .into_boxed_slice(),
    ))
}
