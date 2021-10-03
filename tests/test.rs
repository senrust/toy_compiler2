extern crate compiler;

#[test]
fn add_test() {
    let tokens = compiler::tokenizer::tokenize("tests/add.test");
    let nodes = compiler::token_interpreter::make_nodes(tokens);
    compiler::ast_maker::make_asts(nodes);
}