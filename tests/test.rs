#![allow(dead_code)]
extern crate compiler;

use std::fs;
use std::path::Path;
use std::process::Command;

fn execute_binary(dir: &Path) -> i32 {
    let status = Command::new("sh")
        .arg("-c")
        .arg(dir.join("./a.out"))
        .status()
        .expect("failed to execute binary")
        .code()
        .unwrap();
    status
}

fn make_binary(dir: &Path) {
    Command::new("cc")
        .arg("-o")
        .arg(dir.join("a.out"))
        .arg(dir.join("tmp.s"))
        .output()
        .expect("failed to make binary");
}

#[test]
fn add_test() {
    let dir = Path::new("tests/add");
    let source = dir.join("add.test");
    let _result = fs::read_to_string(dir.join("result"))
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    let tokens = compiler::source_tokenizer::tokenize(&source);
    let nodes = compiler::token_interpreter::make_nodes(tokens);
    compiler::ast_maker::make_asts(nodes);
}
