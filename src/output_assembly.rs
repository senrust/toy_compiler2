use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast_maker::*;
use crate::definition::number::Number;
use crate::error::{unexpected_ast_err, unsupported_ast_err};

fn push_number<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::ImmidiateInterger(Number::U64(num)) = ast.kind {
        writeln!(buf, "    push {}", num).unwrap();
    } else {
        unexpected_ast_err(&ast, "imidiate number".to_string());
    }
}

fn write_pop_two_values<T: Write>(buf: &mut T) {
    writeln!(buf, "    pop rax").unwrap();
    writeln!(buf, "    pop rdi").unwrap();
}

fn write_operation<T: Write>(buf: &mut T, ope: &str) {
    write_pop_two_values(buf);
    writeln!(buf, "    {} rax, rdi", ope).unwrap();
    writeln!(buf, "    push rax").unwrap();
}

fn write_compararison<T: Write>(buf: &mut T, comp: &str) {
    write_pop_two_values(buf);
    writeln!(buf, "    cmp rax, rdi").unwrap();
    writeln!(buf, "    {} al", comp).unwrap();
    writeln!(buf, "    sete al").unwrap();
    writeln!(buf, "    movzb rax, al").unwrap();
    writeln!(buf, "    push rax").unwrap();
}

fn exetute_mul<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::Operation(Operation::Mul) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_operation(buf, "imul");
    } else {
        unexpected_ast_err(&ast, "operation *".to_string());
    }
}

fn exetute_div<T: Write>(ast: &mut AST, buf: &mut T) {
    if let ASTKind::Operation(Operation::Div) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_pop_two_values(buf);
        writeln!(buf, "    cqo").unwrap();
        writeln!(buf, "    idiv rdi").unwrap();
        writeln!(buf, "    push rax").unwrap();
    } else {
        unexpected_ast_err(&ast, "operation /".to_string());
    }
}

fn exetute_add<T: Write>(ast: &mut AST, buf: &mut T) {
    let operation;
    if let ASTKind::Operation(Operation::Add) = ast.kind {
        operation = "add";
    } else if let ASTKind::Operation(Operation::Sub) = ast.kind {
        operation = "sub";
    } else {
        unexpected_ast_err(&ast, "operation + or -".to_string());
    }

    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_operation(buf, operation);
}

fn exetute_eq<T: Write>(ast: &mut AST, buf: &mut T) {
    let euality;
    if let ASTKind::Operation(Operation::Eq) = ast.kind {
        euality = "sete";
    } else if let ASTKind::Operation(Operation::NotEq) = ast.kind {
        euality = "setne";
    } else {
        unexpected_ast_err(&ast, "operation == or !=".to_string());
    }
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, euality);
}

fn exetute_comp<T: Write>(ast: &mut AST, buf: &mut T) {
    let comparison;
    // Gt, Geは右辺と左辺を反転させたLt, Leとして扱う
    if let ASTKind::Operation(Operation::Gt | Operation::Ge) = ast.kind {
        std::mem::swap(&mut ast.right, &mut ast.left);
        if let ASTKind::Operation(Operation::Gt) = ast.kind {
            ast.kind = ASTKind::Operation(Operation::Lt);
        } else {
            ast.kind = ASTKind::Operation(Operation::Le);
        }
    }

    if let ASTKind::Operation(Operation::Lt) = ast.kind {
        comparison = "setl";
    } else if let ASTKind::Operation(Operation::Le) = ast.kind {
        comparison = "setle";
    } else {
        unexpected_ast_err(&ast, "operation >, <, >= or <=".to_string());
    }
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, comparison);
}

fn output_ast<T: Write>(ast: &mut AST, buf: &mut T) {
    match &ast.kind {
        ASTKind::Operation(Operation::Add | Operation::Sub) => exetute_add(ast, buf),
        ASTKind::Operation(Operation::Mul) => exetute_mul(ast, buf),
        ASTKind::Operation(Operation::Div) => exetute_div(ast, buf),
        ASTKind::Operation(Operation::Eq | Operation::NotEq) => exetute_eq(ast, buf),
        ASTKind::Operation(Operation::Gt | Operation::Lt | Operation::Ge | Operation::Le) => {
            exetute_comp(ast, buf)
        }
        ASTKind::ImmidiateInterger(_num) => push_number(ast, buf),
        _ => unsupported_ast_err(&ast),
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
