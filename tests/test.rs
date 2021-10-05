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

fn make_binary(dir: &Path, assembley_path: &Path) {
    let command;
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        // something
        command = "cc";
    } else {
        command = "x86_64-linux-gnu-gcc";
    }

    let sts = Command::new(command)
        .arg("-o")
        .arg(dir.join("a.out"))
        .arg(assembley_path)
        .status()
        .expect("failed to make binary")
        .code()
        .unwrap();
    assert_eq!(0, sts);
}

fn do_compile(dir: &Path, source: &Path, output: &Path) {
    let tokens = compiler::source_tokenizer::tokenize(&source);
    let nodes = compiler::token_interpreter::make_nodes(tokens);
    let asts = compiler::ast_maker::make_asts(nodes);
    compiler::output_assembly::output_assembly(asts, output);
    make_binary(dir, output);
}

#[test]
fn add_test() {
    let dir = Path::new("tests/add");
    let source = dir.join("add.test");
    let output = dir.join("tmp.s");
    let answer = fs::read_to_string(dir.join("result"))
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    do_compile(dir, &source, &output);
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        let result = execute_binary(dir);
        assert_eq!(result, answer);
    }
}
