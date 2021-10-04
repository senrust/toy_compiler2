#![allow(dead_code)]
mod ast_maker;
mod definition;
mod error;
mod output_assembly;
mod source_tokenizer;
mod token_interpreter;

use ast_maker::make_asts;
use source_tokenizer::tokenize;
use std::env;
use std::path::Path;
use token_interpreter::make_nodes;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let args = env::args().collect::<Vec<String>>();
    for arg in args.iter().skip(1) {
        let path = Path::new(arg);
        let tokens = tokenize(path);
        let nodes = make_nodes(tokens);
        let asts = make_asts(nodes);
        let outputpath = Path::new("./tmp.s");
        output_assembly::output_assembly(asts, &outputpath);
    }
}
