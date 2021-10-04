mod definition;
mod error;
mod token_interpreter;
mod ast_maker;
mod tokenizer;

use std::env;
use std::path::Path;
use token_interpreter::make_nodes;
use ast_maker::make_asts;
use tokenizer::tokenize;


static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let args = env::args().collect::<Vec<String>>();
    for arg in args.iter().skip(1) {
        let path = Path::new(arg);
        let tokens = tokenize(path);
        let nodes = make_nodes(tokens);
        make_asts(nodes);

    }
}
