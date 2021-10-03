extern crate compiler;

#[test]
fn add_test() {
    let tokens = compiler::tokenizer::tokenize("tests/add.test");
    let nodes = compiler::interpret_token::make_nodes(tokens);
    compiler::make_ast::make_asts(nodes);
}