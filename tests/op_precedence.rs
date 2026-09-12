use std::{env, fs, path::Path};

use rant::{
    lexer::Token,
    parser::{Declaration, Expression, Factor, FunctionCall, Term},
};

fn lex_op_precedence() -> Vec<Token> {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/op_precedence.rant"))
            .unwrap();

    rant::lexer::tokenize(Box::leak(source.into_boxed_str())).unwrap()
}

fn parse_op_precedence(tokens: Vec<Token>) {
    use rant::parser::{self, Node};

    let parsed_ast = parser::parse(tokens).unwrap();

    let expected_ast = Node::Program(Box::new([
        Node::Declaration(Declaration::FunctionDecl {
            ident: "main".into(),
            param_types: Box::new([]),
            return_type: "Void".into(),
        }),
        Node::Declaration(Declaration::FunctionDef {
            ident: "main".into(),
            param_idents: Box::new([]),
            body: Expression::Term(Term::Factor(Factor::FunctionCall(FunctionCall {
                ident: "print".into(),
                params: Box::new([Expression::Add(
                    Term::Factor(Factor::Number(1)),
                    Box::new(Expression::Sub(
                        Term::Mult(
                            Box::new(Term::Factor(Factor::Number(2))),
                            Box::new(Term::Mult(
                                Box::new(Term::Factor(Factor::Number(2))),
                                Box::new(Term::Factor(Factor::Number(2))),
                            )),
                        ),
                        Box::new(Expression::Term(Term::Factor(Factor::Number(1)))),
                    )),
                )]),
            }))),
        }),
    ]));

    assert_eq!(parsed_ast, expected_ast)
}

#[test]
fn test_op_precedence() {
    let tokens = lex_op_precedence();

    parse_op_precedence(tokens);
}
