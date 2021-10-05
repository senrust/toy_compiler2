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
        unexpected_ast_err(&ast, "imidiate number".to_string());
    }
}

fn write_operation_pop<T: Write>(buf: &mut T) {
    writeln!(buf, "    pop rax").unwrap();
    writeln!(buf, "    pop rdi").unwrap();
}

fn write_operation<T: Write>(buf: &mut T, ope: &str) {
    write_operation_pop(buf);
    writeln!(buf, "    {} rax, rdi", ope).unwrap();
    writeln!(buf, "    push rax").unwrap();
}

fn exetute_mul<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::Operation(Operation::Mul) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_operation(buf, "imul");
    } else {
        unexpected_ast_err(&ast, "operation mul".to_string());
    }
}

fn exetute_div<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::Operation(Operation::Div) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_operation_pop(buf);
        writeln!(buf, "    cqo").unwrap();
        writeln!(buf, "    idiv rdi").unwrap();
        writeln!(buf, "    push rax").unwrap();
    } else {
        unexpected_ast_err(&ast, "operation div".to_string());
    }
}

fn exetute_add<T: Write>(ast: &mut AST, buf: &mut T) {
    let operation;
    if let ASTKind::Operation(Operation::Add) = ast.kind {
        operation = "add";
    } else if let ASTKind::Operation(Operation::Sub) = ast.kind {
        operation = "sub";
    } else {
        unexpected_ast_err(&ast, "operation add or sub".to_string());
    }

    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_operation(buf, operation);
}

fn output_ast<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::Operation(Operation::Add | Operation::Sub) = ast.kind {
        exetute_add(ast, buf);
    } else if let ASTKind::Operation(Operation::Mul) = ast.kind {
        exetute_mul(ast, buf);
    } else if let ASTKind::Operation(Operation::Div) = ast.kind {
        exetute_div(ast, buf);
    } else if let ASTKind::ImmidiateInterger(_) = ast.kind {
        push_number(ast, buf);
    } else {
        unexpected_ast_err(&ast, "operation add or sub".to_string());
    }
}

fn output_asts<T: Write>(asts: Vec<AST>, buf: &mut T) {
    // 現状は1関数のみなのでmainだけ
    for mut ast in asts {
        writeln!(buf, "main:").unwrap();
        output_ast(&mut ast, buf);
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
