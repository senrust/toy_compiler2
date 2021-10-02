mod interpret_token;
mod tokenizer;
mod make_ast;
mod definition;
mod error;

use interpret_token::make_nodes;
use tokenizer::tokenize;
use make_ast::make_asts;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let tokens = tokenize("main.c");
    for token in &tokens {
        println!("{}", token.token);
    }
    let nodes = make_nodes(tokens);
    for node in &nodes {
        println!("{:?}", node.kind);
        println!("{:?}", node.info);
    }
    make_asts(nodes);
}
