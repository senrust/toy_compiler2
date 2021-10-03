mod definition;
mod error;
mod interpret_token;
mod make_ast;
mod tokenizer;

use std::env;
use interpret_token::make_nodes;
use make_ast::make_asts;
use tokenizer::tokenize;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let files = env::args().collect::<Vec<String>>();
    for file in files.iter().skip(1) {
        let tokens = tokenize(file.as_str());
        let nodes = make_nodes(tokens);
        make_asts(nodes);

    }
}
