use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast_maker::*;
use crate::definition::number::Number;
use crate::error::unexpected_ast_err;

fn push_number<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::ImmidiateInterger(Number::U64(num)) = ast.kind {
        writeln!(buf, "    push {}", num).unwrap();
    } else {
        unexpected_ast_err(&ast, "imidiate number");
    }
}

fn exetute_add<T: Write>(ast: &mut AST, buf: &mut T) {
    let operation_str;
    if let ASTKind::Operation(Operation::Add) = ast.kind {
        operation_str = "add";
    } else if let ASTKind::Operation(Operation::Sub) = ast.kind {
        operation_str = "sub";
    } else if let ASTKind::ImmidiateInterger(_) = ast.kind {
        push_number(ast, buf);
        return;
    } else {
        unexpected_ast_err(&ast, "operation add or sub");
    }
    exetute_add(ast.right.take().unwrap().as_mut(), buf);
    exetute_add(ast.left.take().unwrap().as_mut(), buf);
    writeln!(buf, "    pop rax").unwrap();
    writeln!(buf, "    pop rbx").unwrap();
    writeln!(buf, "    {} rax, rbx", operation_str).unwrap();
    writeln!(buf, "    push rax").unwrap();
}

fn output_ast<T: Write>(mut ast: AST, buf: &mut T) {
    exetute_add(&mut ast, buf);
}

fn output_asts<T: Write>(asts: Vec<AST>, buf: &mut T) {
    // 現状は1関数のみなのでmainだけ
    for ast in asts {
        writeln!(buf, "main:").unwrap();
        output_ast(ast, buf);
        writeln!(buf, "    pop rax").unwrap();
        writeln!(buf, "    ret").unwrap();
    }
}

fn write_assembly_header<T: Write>(buf: &mut T) {
    writeln!(buf, ".intel_syntax noprefix").unwrap();
    writeln!(buf, ".globl main").unwrap();
    writeln!(buf, "").unwrap();
}

pub fn output_assembly(asts: Vec<AST>, output_file: &Path) {
    let mut buf = BufWriter::new(fs::File::create(output_file).unwrap());
    write_assembly_header(&mut buf);
    output_asts(asts, &mut buf);
}
