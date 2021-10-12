use std::io::Write;

use crate::ast::ast::*;
use crate::ast::error::*;
use crate::output::output::*;

pub fn write_operation<T: Write>(buf: &mut OutputBuffer<T>, ope: &str) {
    write_pop_two_values(buf);
    let instruction = format!("    {} rax, rdi", ope);
    buf.output(&instruction);
    buf.output_push("rax");
}

fn write_compararison<T: Write>(buf: &mut OutputBuffer<T>, comp_type: &str) {
    write_pop_two_values(buf);
    let comp_output = format!("    {} al", comp_type);
    buf.output("    cmp rax, rdi");
    buf.output(&comp_output);
    buf.output("    movzb rax, al");
    buf.output_push("rax");
}

fn write_value_compararison<T: Write>(buf: &mut OutputBuffer<T>, comp_type: &str, num: usize) {
    write_pop_one_value(buf);
    let comp_instruction = format!("    cmp rax, {}", num);
    let comp_output = format!("    {} al", comp_type);
    buf.output(&comp_instruction);
    buf.output(&comp_output);
    buf.output("    movzb rax, al");
    buf.output_push("rax");
}

fn write_assignment<T: Write>(buf: &mut OutputBuffer<T>) {
    write_pop_two_values(buf);
    buf.output("    mov [rax], rdi");
    buf.output_push("rdi");
}

fn exetute_mul<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::Operation(Operation::Mul) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_operation(buf, "imul");
    } else {
        unexpected_ast_err(ast, "operation *");
    }
}

fn exetute_div<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::Operation(Operation::Div | Operation::Rem) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_pop_two_values(buf);
        buf.output("    cqo");
        buf.output("    idiv rdi");
        if ast.kind == AstKind::Operation(Operation::Div) {
            buf.output_push("rax");
        } else {
            buf.output_push("rdx");
        }
    } else {
        unexpected_ast_err(ast, "operation / or %");
    }
}

fn exetute_add<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let operation = match ast.kind {
        AstKind::Operation(Operation::Add) => "add",
        AstKind::Operation(Operation::Sub) => "sub",
        _ => unexpected_ast_err(ast, "operation + or -"),
    };

    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_operation(buf, operation);
}

fn exetute_eq<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let euality = match ast.kind {
        AstKind::Operation(Operation::Eq) => "sete",
        AstKind::Operation(Operation::NotEq) => "setne",
        _ => unexpected_ast_err(ast, "operation == or !="),
    };
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, euality);
}

fn exetute_comp<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    // Gt, Geは右辺と左辺を反転させたLt, Leとして扱う
    if let AstKind::Operation(Operation::Gt | Operation::Ge) = ast.kind {
        std::mem::swap(&mut ast.right, &mut ast.left);
        if let AstKind::Operation(Operation::Gt) = ast.kind {
            ast.kind = AstKind::Operation(Operation::Lt);
        } else {
            ast.kind = AstKind::Operation(Operation::Le);
        }
    }

    let comparison = match ast.kind {
        AstKind::Operation(Operation::Lt) => "setl",
        AstKind::Operation(Operation::Le) => "setle",
        _ => unexpected_ast_err(ast, "operation >, <, >= or <="),
    };
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, comparison);
}

fn exetute_not<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    output_ast(ast.operand.take().unwrap().as_mut(), buf);
    write_value_compararison(buf, "sete", 0);
}

fn exetute_assign<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    // 左辺値が被代入可能か確認
    let left_ast = ast.left.take().unwrap();
    if let AstKind::Variable(_val) = &left_ast.kind {
        // 代入は右から評価する
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        push_variable_address(left_ast.as_ref(), buf);
        write_assignment(buf);
    } else {
        unassignable_ast_err(ast);
    }
}

fn exetute_bit_operation<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let bit_operation = match ast.kind {
        AstKind::Operation(Operation::BitAnd) => "and",
        AstKind::Operation(Operation::BitOr) => "or",
        AstKind::Operation(Operation::BitXor) => "xor",
        _ => unexpected_ast_err(ast, "operation &, | or ^"),
    };

    output_ast(ast.left.take().unwrap().as_mut(), buf);
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    write_operation(buf, bit_operation);
}

pub fn output_operation_ast<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(Operation::Add | Operation::Sub) => exetute_add(ast, buf),
        AstKind::Operation(Operation::Mul) => exetute_mul(ast, buf),
        AstKind::Operation(Operation::Div | Operation::Rem) => exetute_div(ast, buf),
        AstKind::Operation(Operation::Eq | Operation::NotEq) => exetute_eq(ast, buf),
        AstKind::Operation(Operation::Gt | Operation::Lt | Operation::Ge | Operation::Le) => {
            exetute_comp(ast, buf)
        }
        AstKind::Operation(Operation::Not) => exetute_not(ast, buf),
        AstKind::Operation(Operation::Assign) => exetute_assign(ast, buf),
        AstKind::Operation(Operation::BitAnd | Operation::BitOr | Operation::BitXor) => {
            exetute_bit_operation(ast, buf)
        }
        _ => unsupported_ast_err(ast),
    }
}
