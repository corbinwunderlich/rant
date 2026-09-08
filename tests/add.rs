use std::{env, fs, path::Path};

use rant::{
    lexer::Token,
    parser::{Declaration, Expression, FunctionCall, Literal, Term, TypedIdent},
};

fn lex_add() -> Vec<Token> {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/add.rant")).unwrap();

    rant::lexer::tokenize(Box::leak(source.into_boxed_str())).unwrap()
}

fn parse_add(tokens: Vec<Token>) {
    use rant::parser::{self, Node};

    let parsed_ast = parser::parse(tokens).unwrap();

    let expected_ast = Node::Program(Box::new([
        Node::Declaration(Declaration::Function {
            ident: "add".into(),
            params: Box::new([
                TypedIdent("a".into(), "Int".into()),
                TypedIdent("b".into(), "Int".into()),
            ]),
            return_type: "Int".into(),
            body: Expression::Add(
                Box::new(Expression::Term(Term::Ident("a".into()))),
                Box::new(Expression::Term(Term::Ident("b".into()))),
            ),
        }),
        Node::Declaration(Declaration::Function {
            ident: "main".into(),
            params: Box::new([]),
            return_type: "Void".into(),
            body: Expression::Term(Term::FunctionCall(FunctionCall(
                "print".into(),
                Box::new([Expression::Term(Term::FunctionCall(FunctionCall(
                    "add".into(),
                    Box::new([
                        Expression::Term(Term::Literal(Literal::Number(1))),
                        Expression::Term(Term::Literal(Literal::Number(2))),
                    ]),
                )))]),
            ))),
        }),
    ]));

    assert_eq!(parsed_ast, expected_ast)
}

#[test]
fn test_add() {
    let tokens = lex_add();

    parse_add(tokens);
}
