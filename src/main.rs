mod definition;
mod error;
mod token_interpreter;
mod ast_maker;
mod tokenizer;

use std::env;
use token_interpreter::make_nodes;
use ast_maker::make_asts;
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
