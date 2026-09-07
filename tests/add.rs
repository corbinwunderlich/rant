use std::{env, fs, path::Path};

#[test]
fn test_lex_add() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/add.rant")).unwrap();

    rant::lexer::tokenize(&source).unwrap();
}
