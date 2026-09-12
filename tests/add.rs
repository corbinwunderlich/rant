use std::{env, fs, path::Path};

use rant::{
    lexer::Token,
    parser::{
        self, Declaration, Expression, Factor, FunctionCall, FunctionDecl, FunctionDef,
        LetBindings, Program, Term,
    },
};

fn lex_add() -> Vec<Token> {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/add.rant")).unwrap();

    rant::lexer::tokenize(Box::leak(source.into_boxed_str())).unwrap()
}

fn parse_add(tokens: Vec<Token>) {
    let parsed_ast = parser::parse(tokens).unwrap();

    let expected_ast = Program {
        nodes: Box::new([
            Declaration::FunctionDecl(FunctionDecl {
                ident: "add".into(),
                param_types: Box::new(["Int".into(), "Int".into()]),
                return_type: "Int".into(),
            }),
            Declaration::FunctionDef(FunctionDef {
                ident: "add".into(),
                param_idents: Box::new(["a".into(), "b".into()]),
                body: Expression::Let(
                    LetBindings {
                        declarations: Box::new([("ans".into(), "Int".into())]),
                        definitions: Box::new([(
                            "ans".into(),
                            Expression::Add(
                                Term::Factor(Factor::Ident("a".into())),
                                Box::new(Expression::Term(Term::Factor(Factor::Ident("b".into())))),
                            ),
                        )]),
                    },
                    Box::new(Expression::Term(Term::Factor(Factor::Ident("ans".into())))),
                ),
            }),
            Declaration::FunctionDecl(FunctionDecl {
                ident: "main".into(),
                param_types: Box::new([]),
                return_type: "Void".into(),
            }),
            Declaration::FunctionDef(FunctionDef {
                ident: "main".into(),
                param_idents: Box::new([]),
                body: Expression::Term(Term::Factor(Factor::FunctionCall(FunctionCall {
                    ident: "print".into(),
                    params: Box::new([Expression::Term(Term::Factor(Factor::FunctionCall(
                        FunctionCall {
                            ident: "add".into(),
                            params: Box::new([
                                Expression::Term(Term::Factor(Factor::Number(1))),
                                Expression::Term(Term::Factor(Factor::Number(2))),
                            ]),
                        },
                    )))]),
                }))),
            }),
        ]),
    };

    assert_eq!(parsed_ast, expected_ast)
}

#[test]
fn test_add() {
    let tokens = lex_add();

    parse_add(tokens);
}
