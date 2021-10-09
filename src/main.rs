#![allow(dead_code)]
mod ast;
mod definition;
mod output;
mod token;

use std::env;
use std::path::Path;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        eprintln!("error: no input files");
        std::process::exit(-1);
    }
    for arg in args.iter().skip(1) {
        let path = Path::new(arg);
        let rawtokens = token::parser::parse_file(path);
        let tokens = token::token::make_tokens(rawtokens);
        let asts = ast::ast::make_asts(tokens);
        let outputpath = Path::new("./tmp.s");
        output::output::output_assembly(asts, outputpath);
    }
}
