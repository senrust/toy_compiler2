use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast::ast::*;
use crate::ast::error::*;
use crate::definition::number::Number;
use crate::definition::variables::*;
use crate::output::controls::*;

#[derive(PartialEq, Debug)]
pub enum LoopKind {
    For,
    While,
}

pub struct OutputBuffer<T: Write> {
    buf: T,
    pub label_index: usize,
    stack_alignment: i32,
    break_info: Vec<(usize, LoopKind)>,
}

impl<T: Write> OutputBuffer<T> {
    fn new(buf: T) -> Self {
        Self {
            buf,
            label_index: 0,
            stack_alignment: 0, // 関数呼び出し用に使用中のスタックサイズを把握する
            // 関数呼び出し時はスタックのアライメントが16バイトである必要があるため,
            // 16バイトアライメントでの位置を記録する
            // stack_alignment = 4　ならば, 関数呼び出し時は 16 -4 = 12 バイト,
            // スタックを増やす必要がある
            break_info: vec![],
        }
    }

    pub fn enter_loop_control(&mut self, type_: LoopKind) {
        self.break_info.push((self.label_index, type_));
    }

    pub fn exit_loop_control(&mut self) {
        self.break_info.pop();
    }

    pub fn get_break_label(&mut self) -> Result<String, ()> {
        if let Some((label_index, type_)) = self.break_info.last() {
            if *type_ == LoopKind::For {
                Ok(format!("    jmp .LabelForEnd{}", label_index))
            } else {
                Ok(format!("    jmp .LabelWhileEnd{}", label_index))
            }
        } else {
            Err(())
        }
    }

    pub fn get_label_index(&self) -> usize {
        self.label_index
    }

    pub fn increment_label(&mut self) {
        self.label_index += 1;
    }

    #[inline]
    pub fn output(&mut self, line: &str) {
        writeln!(self.buf, "{}", line).unwrap();
    }

    #[inline]
    pub fn output_push(&mut self, register: &str) {
        writeln!(self.buf, "    push {}", register).unwrap();
        self.stack_alignment = (self.stack_alignment + 8) / 16;
    }

    #[inline]
    fn output_push_num(&mut self, num: u64) {
        // 即値は4ビット
        writeln!(self.buf, "    push {}", num).unwrap();
        self.stack_alignment = (self.stack_alignment + 4) / 16;
    }

    #[inline]
    pub fn output_pop(&mut self, register: &str) {
        writeln!(self.buf, "    pop {}", register).unwrap();
        // 16バイトアライメントなので正の値とする
        let mut rem = self.stack_alignment - 8;
        if rem < 0 {
            rem += 16;
        }
        self.stack_alignment = rem;
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

pub fn output_function_prelude<T: Write>(
    func_name: &str,
    local_val_size: &usize,
    buf: &mut OutputBuffer<T>,
) {
    let func_label = &format!("{}:", func_name);
    buf.output(func_label);
    buf.output_push("rbp");
    buf.output("    mov rbp, rsp");
    // ローカル変数を使用するときのみ
    // rbpの退避分でスタックは8バイト使用している
    if *local_val_size > 8 {
        let sub_rsp_instruction = &format!("    sub rsp, {}", local_val_size - 8);
        buf.output(sub_rsp_instruction);
    }
}

pub fn output_function_epilogue<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output("    mov rsp, rbp");
    buf.output_pop("rbp");
    buf.output("    ret");
}

fn push_number<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::ImmidiateInterger(Number::U64(num)) = ast.kind {
        buf.output_push_num(num)
    } else {
        unexpected_ast_err(ast, "imidiate number");
    }
}

fn push_variable_value<T: Write>(ast: &Ast, buf: &mut OutputBuffer<T>) {
    push_variable_address(ast, buf);
    buf.output_pop("rax");
    buf.output("    mov rax, [rax]");
    buf.output_push("rax");
}

fn push_variable_address<T: Write>(ast: &Ast, buf: &mut OutputBuffer<T>) {
    // 現在はローカル変数のみ対応
    if let AstKind::Variable(Variable::LocalVal(local_val)) = &ast.kind {
        buf.output("    mov rax, rbp");
        let local_offset = format!("    sub rax, {}", local_val.frame_offset);
        buf.output(&local_offset);
        buf.output_push("rax");
    } else {
        unexpected_ast_err(ast, "local variable");
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
    if let AstKind::Operation(Operation::Div) = ast.kind {
        output_ast(ast.right.take().unwrap().as_mut(), buf);
        output_ast(ast.left.take().unwrap().as_mut(), buf);
        write_pop_two_values(buf);
        buf.output("    cqo");
        buf.output("    idiv rdi");
        buf.output_push("rax");
    } else {
        unexpected_ast_err(ast, "operation /");
    }
}

fn exetute_add<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let operation;
    if let AstKind::Operation(Operation::Add) = ast.kind {
        operation = "add";
    } else if let AstKind::Operation(Operation::Sub) = ast.kind {
        operation = "sub";
    } else {
        unexpected_ast_err(ast, "operation + or -");
    }

    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_operation(buf, operation);
}

fn exetute_eq<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let euality;
    if let AstKind::Operation(Operation::Eq) = ast.kind {
        euality = "sete";
    } else if let AstKind::Operation(Operation::NotEq) = ast.kind {
        euality = "setne";
    } else {
        unexpected_ast_err(ast, "operation == or !=");
    }
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, euality);
}

fn exetute_comp<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let comparison;
    // Gt, Geは右辺と左辺を反転させたLt, Leとして扱う
    if let AstKind::Operation(Operation::Gt | Operation::Ge) = ast.kind {
        std::mem::swap(&mut ast.right, &mut ast.left);
        if let AstKind::Operation(Operation::Gt) = ast.kind {
            ast.kind = AstKind::Operation(Operation::Lt);
        } else {
            ast.kind = AstKind::Operation(Operation::Le);
        }
    }

    if let AstKind::Operation(Operation::Lt) = ast.kind {
        comparison = "setl";
    } else if let AstKind::Operation(Operation::Le) = ast.kind {
        comparison = "setle";
    } else {
        unexpected_ast_err(ast, "operation >, <, >= or <=");
    }
    output_ast(ast.right.take().unwrap().as_mut(), buf);
    output_ast(ast.left.take().unwrap().as_mut(), buf);
    write_compararison(buf, comparison);
}

fn exetute_not<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Operation(Operation::Not) => (),
        _ => unexpected_ast_err(ast, "operation !"),
    }

    output_ast(ast.operand.take().unwrap().as_mut(), buf);
    write_value_compararison(buf, "sete", 0);
}

fn exetute_assign<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Operation(Operation::Assign) => (),
        _ => unexpected_ast_err(ast, "operation ="),
    }
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

// 複文のコンパイル
fn excute_exprs<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Expressions => (),
        _ => unexpected_ast_err(ast, "{} block"),
    }
    let expr_ast_vec = ast.exprs.take().unwrap();
    for mut expr_ast in expr_ast_vec {
        output_ast(&mut expr_ast, buf);
        // 複文側の最後, 各制御文側でpopしているのでこちらではpopしない
        // if文やfor文の{}後も複文の制御構文側でpopしているのでこちらでは行わない
        if !matches!(
            expr_ast.kind,
            AstKind::Expressions | AstKind::Control(Control::For | Control::If | Control::While)
        ) {
            buf.output_pop("rax");
        }
    }
}

pub fn output_expr_ast<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(_) => {
            output_operation_ast(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Control(_) => output_control_ast(ast, buf),
        AstKind::Expressions => excute_exprs(ast, buf),
        _ => unsupported_ast_err(ast),
    }
}

pub fn output_operation_ast<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(Operation::Add | Operation::Sub) => exetute_add(ast, buf),
        AstKind::Operation(Operation::Mul) => exetute_mul(ast, buf),
        AstKind::Operation(Operation::Div) => exetute_div(ast, buf),
        AstKind::Operation(Operation::Eq | Operation::NotEq) => exetute_eq(ast, buf),
        AstKind::Operation(Operation::Gt | Operation::Lt | Operation::Ge | Operation::Le) => {
            exetute_comp(ast, buf)
        }
        AstKind::Operation(Operation::Not) => exetute_not(ast, buf),
        AstKind::Operation(Operation::Assign) => exetute_assign(ast, buf),
        _ => unsupported_ast_err(ast),
    }
}

pub fn output_control_ast<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Control(Control::Return) => execute_return(ast, buf),
        AstKind::Control(Control::If) => execute_if(ast, buf),
        AstKind::Control(Control::For) => execute_for(ast, buf),
        AstKind::Control(Control::While) => execute_while(ast, buf),
        AstKind::Control(Control::Break) => execute_break(ast, buf),
        _ => unsupported_ast_err(ast),
    }
}

pub fn output_ast<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(_) => output_operation_ast(ast, buf),
        AstKind::Control(_) => output_control_ast(ast, buf),
        AstKind::ImmidiateInterger(_num) => push_number(ast, buf),
        AstKind::Variable(_val) => push_variable_value(ast, buf),
        AstKind::Expressions => excute_exprs(ast, buf),
        _ => unsupported_ast_err(ast),
    }
}

fn output_function<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    // 現状は1関数のみなのでmainだけ
    match &ast.kind {
        AstKind::FunctionImplementation((func_name, local_val_size)) => {
            output_function_prelude(func_name, local_val_size, buf);
            let mut func_context_ast = ast.context.take().unwrap();
            output_ast(func_context_ast.as_mut(), buf);
            output_function_epilogue(buf);
        }
        _ => unsupported_ast_err(ast),
    }
}

fn write_assembly_header<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output(".intel_syntax noprefix");
    buf.output(".globl main");
    buf.output("");
}

pub fn output_assembly(asts: Vec<Ast>, output_file: &Path) {
    let buf = BufWriter::new(fs::File::create(output_file).unwrap());
    let mut outputbuf = OutputBuffer::new(buf);
    write_assembly_header(&mut outputbuf);
    for mut ast in asts {
        output_function(&mut ast, &mut outputbuf);
    }
}