mod interpret_token;
mod make_ast;
mod tokenizer;

use interpret_token::make_ast_nodes;
use tokenizer::tokenize;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let tokens = tokenize("main.c");
    for token in &tokens {
        println!("{}", token.token);
    }
    let ast_nodes = make_ast_nodes(tokens);
    for node in &ast_nodes {
        println!("{:?}", node.kind);
        println!("{:?}", node.info);
    }
}
