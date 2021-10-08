extern crate compiler;

use std::fs;
use std::path::{Path, PathBuf};
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
    let tokens = compiler::source_tokenizer::tokenize(source);
    let nodes = compiler::token_interpreter::make_nodes(tokens);
    let asts = compiler::ast_maker::make_asts(nodes);
    compiler::output_assembly::output_assembly(asts, output);
    make_binary(dir, output);
}

fn get_test_parameter(test_type: &str) -> (PathBuf, PathBuf, PathBuf, i32) {
    let dir = Path::new("tests").join(test_type);
    let source = dir.join(format!("{}.test", test_type));
    let output = dir.join("tmp.s");
    let answer = fs::read_to_string(dir.join("result"))
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    (dir, source, output, answer)
}

fn do_test(test_type: &str) {
    let (dir, source, output, answer) = get_test_parameter(test_type);
    do_compile(&dir, &source, &output);
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        let result = execute_binary(&dir);
        assert_eq!(result, answer);
    }
}

#[test]
fn add_test() {
    do_test("add");
}

#[test]
fn mul_test() {
    do_test("mul");
}

#[test]
fn parenthesis_test() {
    do_test("parenthesis");
}

#[test]
fn unary_test() {
    do_test("unary");
}

#[test]
fn equality_test() {
    do_test("equality");
}

#[test]
fn relational_test() {
    do_test("relational");
}

#[test]
fn not_test() {
    do_test("not");
}

#[test]
fn variable_test() {
    // 未初期化のローカル変数を使うのでコンパイルのみ行う
    let (dir, source, output, _answer) = get_test_parameter("variable");
    do_compile(&dir, &source, &output);
}

#[test]
fn assign_test() {
    do_test("assign");
}

#[test]
fn expressions_test() {
    do_test("expressions");
}

#[test]
fn brackets_test() {
    do_test("brackets");
}
