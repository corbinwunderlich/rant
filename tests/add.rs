use std::{env, fs, path::Path};

use rant::{
    lexer::Token,
    parser::{Declaration, Expression, FunctionCall, LetIn, Literal, Term},
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
        Node::Declaration(Declaration::FunctionDeclaration {
            ident: "add".into(),
            param_types: Box::new(["Int".into(), "Int".into()]),
            return_type: "Int".into(),
        }),
        Node::Declaration(Declaration::FunctionDefinition {
            ident: "add".into(),
            param_idents: Box::new(["a".into(), "b".into()]),
            body: Expression::LetIn(
                LetIn {
                    declarations: Box::new([("ans".into(), "Int".into())]),
                    definitions: Box::new([(
                        "ans".into(),
                        Expression::Addition(
                            Box::new(Expression::Term(Term::Ident("a".into()))),
                            Box::new(Expression::Term(Term::Ident("b".into()))),
                        ),
                    )]),
                },
                Box::new(Expression::Term(Term::Ident("ans".into()))),
            ),
        }),
        Node::Declaration(Declaration::FunctionDeclaration {
            ident: "main".into(),
            param_types: Box::new([]),
            return_type: "Void".into(),
        }),
        Node::Declaration(Declaration::FunctionDefinition {
            ident: "main".into(),
            param_idents: Box::new([]),
            body: Expression::Term(Term::FunctionCall(FunctionCall {
                ident: "print".into(),
                params: Box::new([Expression::Term(Term::FunctionCall(FunctionCall {
                    ident: "add".into(),
                    params: Box::new([
                        Expression::Term(Term::Literal(Literal::Number(1))),
                        Expression::Term(Term::Literal(Literal::Number(2))),
                    ]),
                }))]),
            })),
        }),
    ]));

    assert_eq!(parsed_ast, expected_ast)
}

#[test]
fn test_add() {
    let tokens = lex_add();

    parse_add(tokens);
}
