use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast_maker::*;
use crate::definition::number::Number;
use crate::definition::variables::*;
use crate::error::*;

struct OutputBuffer<T: Write> {
    buf: T,
    stack_alignment: i32,
}

impl<T: Write> OutputBuffer<T> {
    fn new(buf: T) -> Self {
        Self {
            buf,
            stack_alignment: 0,
        }
    }

    #[inline]
    fn output(&mut self, line: &str) {
        writeln!(self.buf, "{}", line).unwrap();
    }

    #[inline]
    fn output_push(&mut self, register: &str) {
        writeln!(self.buf, "    push {}", register).unwrap();
        self.stack_alignment = (self.stack_alignment + 8) / 16;
    }

    #[inline]
    fn output_push_num(&mut self, num: u64) {
        writeln!(self.buf, "    push {}", num).unwrap();
        self.stack_alignment = (self.stack_alignment + 8) / 16;
    }

    #[inline]
    fn output_pop(&mut self, register: &str) {
        writeln!(self.buf, "    pop {}", register).unwrap();
        self.stack_alignment = (self.stack_alignment - 8) / 16;
    }
}

impl<T: Write> Write for OutputBuffer<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.flush()
    }
}

fn output_function_prelude<T: Write>(
    func_name: &str,
    local_val_size: &usize,
    buf: &mut OutputBuffer<T>,
) {
    let ref func_label = format!("{}:", func_name);
    buf.output(func_label);
    buf.output_push("rbp");
    buf.output("    mov rbp, rsp");
    // ローカル変数を使用するときのみ
    // rbpの退避分でスタックは8バイト使用している
    if *local_val_size > 8 {
        let ref sub_rsp_instruction = format!("    sub rsp, {}", local_val_size - 8);
        buf.output(sub_rsp_instruction);
    }
}

fn output_function_epilogue<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output("    mov rsp, rbp");
    buf.output_pop("rbp");
    buf.output("ret");
    buf.output("");
}

fn push_number<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    if let ASTKind::ImmidiateInterger(Number::U64(num)) = ast.kind {
        buf.output_push_num(num)
    } else {
        unexpected_ast_err(&ast, "imidiate number".to_string());
    }
}

fn push_variable_value<T: Write>(ast: &AST, buf: &mut OutputBuffer<T>) {
    push_variable_address(ast, buf);
    buf.output_pop("rax");
    buf.output("    mov rax, [rax]");
    buf.output_push("rax");
}

fn push_variable_address<T: Write>(ast: &AST, buf: &mut OutputBuffer<T>) {
    // 現在はローカル変数のみ対応
    if let ASTKind::Variable(Variable::LocalVal(local_val)) = &ast.kind {
        buf.output("    mov rax, rbp");
        let local_offset = format!("    sub rax, {}", local_val.frame_offset);
        buf.output(&local_offset);
        buf.output_push("rax");
    } else {
        unexpected_ast_err(&ast, "local variable".to_string());
    }
}

#[inline]
fn write_pop_one_value<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output_pop("rax");
}

#[inline]
fn write_pop_two_values<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output_pop("rax");
    buf.output_pop("rdi");
}

fn write_operation<T: Write>(buf: &mut OutputBuffer<T>, ope: &str) {
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

fn exetute_mul<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    if let ASTKind::Operation(Operation::Mul) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_operation(buf, "imul");
    } else {
        unexpected_ast_err(&ast, "operation *".to_string());
    }
}

fn exetute_div<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    if let ASTKind::Operation(Operation::Div) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_pop_two_values(buf);
        buf.output("    cqo");
        buf.output("    idiv rdi");
        buf.output_push("rax");
    } else {
        unexpected_ast_err(&ast, "operation /".to_string());
    }
}

fn exetute_add<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
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

fn exetute_eq<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
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

fn exetute_comp<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
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

fn exetute_not<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        ASTKind::Operation(Operation::Not) => (),
        _ => unexpected_ast_err(&ast, "operation !".to_string()),
    }

    output_ast(ast.operand.take().unwrap().as_mut(), buf);
    write_value_compararison(buf, "sete", 0);
}

fn exetute_assign<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        ASTKind::Operation(Operation::Assign) => (),
        _ => unexpected_ast_err(&ast, "operation =".to_string()),
    }
    // 左辺値が被代入可能化確認
    let left_ast = ast.left.take().unwrap();
    if let ASTKind::Variable(_val) = &left_ast.kind {
        // 代入は右から評価する
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        push_variable_address(left_ast.as_ref(), buf);
        write_assignment(buf);
        return;
    } else {
        unassignable_ast_err(&ast);
    }
}

// 複文のコンパイル
fn excute_exprs<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        ASTKind::Expressions => (),
        _ => unexpected_ast_err(&ast, "{} block".to_string()),
    }
    let expr_ast_vec = ast.exprs.take().unwrap();
    for mut expr_ast in expr_ast_vec {
        output_ast(&mut expr_ast, buf);
        buf.output_pop("rax");
    }

    // 複文の最後にカンマがない文が存在する場合
    if let Some(ast) = ast.context.take().as_mut() {
        output_ast(ast, buf);
    }
}

fn output_ast<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        ASTKind::Operation(Operation::Add | Operation::Sub) => exetute_add(ast, buf),
        ASTKind::Operation(Operation::Mul) => exetute_mul(ast, buf),
        ASTKind::Operation(Operation::Div) => exetute_div(ast, buf),
        ASTKind::Operation(Operation::Eq | Operation::NotEq) => exetute_eq(ast, buf),
        ASTKind::Operation(Operation::Gt | Operation::Lt | Operation::Ge | Operation::Le) => {
            exetute_comp(ast, buf)
        }
        ASTKind::Operation(Operation::Not) => exetute_not(ast, buf),
        ASTKind::Operation(Operation::Assign) => exetute_assign(ast, buf),
        ASTKind::ImmidiateInterger(_num) => push_number(ast, buf),
        ASTKind::Variable(_val) => push_variable_value(ast, buf),
        ASTKind::Expressions => excute_exprs(ast, buf),
        _ => unsupported_ast_err(&ast),
    }
}

fn output_function<T: Write>(ast: &mut AST, buf: &mut OutputBuffer<T>) {
    // 現状は1関数のみなのでmainだけ
    match &ast.kind {
        ASTKind::FunctionImplementation((func_name, local_val_size)) => {
            output_function_prelude(func_name, local_val_size, buf);
            let mut func_context_ast = ast.context.take().unwrap();
            output_ast(func_context_ast.as_mut(), buf);
            output_function_epilogue(buf);
        }
        _ => unsupported_ast_err(&ast),
    }
}

fn write_assembly_header<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output(".intel_syntax noprefix");
    buf.output(".globl main");
    buf.output("");
}

pub fn output_assembly(asts: Vec<AST>, output_file: &Path) {
    let buf = BufWriter::new(fs::File::create(output_file).unwrap());
    let mut outputbuf = OutputBuffer::new(buf);
    write_assembly_header(&mut outputbuf);
    for mut ast in asts {
        output_function(&mut ast, &mut outputbuf);
    }
}
