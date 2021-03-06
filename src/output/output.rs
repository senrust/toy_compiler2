use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast::ast::*;
use crate::ast::error::*;
use crate::definition::number::Number;
use crate::definition::variables::*;
use crate::output::controls::*;
use crate::output::operations::*;

pub const FUNC_ARG_REGISTERS: [&str; 5] = ["rdi", "rdx", "rcx", "r8", "r9"];

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
    pub fn output_push_num(&mut self, num: u64) {
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
    buf.output("");
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

fn push_number<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::ImmidiateInterger(Number::U64(num)) = ast.kind {
        buf.output_push_num(num)
    } else {
        unexpected_ast_err(&ast, "imidiate number");
    }
}

pub fn push_variable_value<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::Variable(Variable::LocalVal(local_val)) = &ast.kind {
        let offset = local_val.frame_offset;
        buf.output(&format!("    push [rbp - {}]", offset));
    }
}

pub fn push_variable_address<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    // 現在はローカル変数のみ対応
    if let AstKind::Variable(Variable::LocalVal(local_val)) = &ast.kind {
        let lea_instruction = format!("    lea rax, [rbp - {}]", local_val.frame_offset);
        buf.output(&lea_instruction);
        buf.output_push("rax");
    } else {
        unexpected_ast_err(&ast, "local variable");
    }
}

// ポインターが指すアドレスを求める
// long **a に対して,
// **aのアドレスは
// aのアドレスを積み,
// aのアドレスの指す値を取り出すを2回繰り返す
pub fn push_pointer_address<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    let mut deref_ast = ast;
    let mut deref_count = 0;
    while let AstKind::Deref = &deref_ast.kind {
        deref_ast = *deref_ast.operand.unwrap();
        deref_count += 1;
    }
    let val_ast = deref_ast;
    push_variable_address(val_ast, buf);
    while deref_count != 0 {
        buf.output_pop("rax");
        buf.output("    mov rax, [rax]");
        buf.output_push("rax");
        deref_count -= 1;
    }
}

// ポインターが指すアドレスの値を求める
// ポインタが指すアドレスの値を取る
pub fn push_deref_value<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    push_pointer_address(ast, buf);
    buf.output_pop("rax");
    buf.output("    mov rax, [rax]");
    buf.output_push("rax");
}

// 配列が指すアドレスを求める
// long a[5][10]の場合,
// a[2][3]へのアクセスでは
// deref(deref(a, 2, size=80), 3, size=8)となっている
// そこで 3 * 8をスタックに積み,
// 2*80をスタックに積む,
// 最後aのアドレスをスタックに積んで,
// a + 160 をスタックに積む
// そして a + 24 をスタックに積む
pub fn push_array_elem_address<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    let mut index_ast = ast;
    let mut indexing_times = 0;
    while let AstKind::Index = &index_ast.kind {
        // index番号をスタックに積む
        let index_num_ast = index_ast.right.take().unwrap();
        output_ast(*index_num_ast, buf);
        // その配列のサイズをスタックに積む
        buf.output_push_num(index_ast.type_.size as u64);
        // サイズ×index番号でオフセットを求めスタックに積む
        write_operation(buf, "imul");
        index_ast = *index_ast.left.unwrap();
        indexing_times += 1;
    }
    //  indexのあとはindex_astは変数なので, この変数のアドレスを取得する
    let val_ast = index_ast;
    push_variable_address(val_ast, buf);
    // あとはオフセットを引く
    for _ in 0..indexing_times {
        write_operation(buf, "sub");
    }
}

pub fn push_array_elem_value<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    push_array_elem_address(ast, buf);
    buf.output_pop("rax");
    buf.output("    mov rax, [rax]");
    buf.output_push("rax");
}

// アドレスを取得する
// アドレスを取得できるのは変数型と, プリミティブ型を返す演算のみ
// AST作成時にチェック済み
pub fn push_address<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let address_ast = ast.operand.take().unwrap();
    match &address_ast.kind {
        AstKind::Address => {
            unaddressable_ast_err(&address_ast);
        }
        AstKind::Variable(_val) => {
            push_variable_address(*address_ast, buf);
        }
        AstKind::Deref => {
            push_pointer_address(*address_ast, buf);
        }
        AstKind::Index => {
            push_array_elem_address(*address_ast, buf);
        }
        _ => output_ast(ast, buf),
    }
}

#[inline]
pub fn write_pop_one_value<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output_pop("rax");
}

#[inline]
pub fn write_pop_two_values<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output_pop("rax");
    buf.output_pop("rdi");
}

// 複文のコンパイル
#[allow(clippy::branches_sharing_code)]
fn excute_exprs<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let expr_ast_vec = ast.exprs.take().unwrap();
    for expr_ast in expr_ast_vec {
        // 複文側の最後, 各制御文側でpopしているのでこちらではpopしない
        // if文やfor文の{}後も複文の制御構文側でpopしているのでこちらでは行わない
        if matches!(
            &expr_ast.kind,
            AstKind::Expressions | AstKind::Control(Control::For | Control::If | Control::While)
        ) {
            output_ast(expr_ast, buf);
        } else {
            output_ast(expr_ast, buf);
            buf.output_pop("rax");
        }
    }
}

// 式のコンパイル
// スタックには値を積まない
pub fn output_formula_ast<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(_) => {
            output_operation_ast(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Control(_) => output_control_ast(ast, buf),
        AstKind::ImmidiateInterger(_num) => {
            push_number(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Variable(_val) => {
            push_variable_value(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Address => {
            push_address(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Deref => {
            push_deref_value(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Index => {
            push_array_elem_value(ast, buf);
            buf.output_pop("rax");
        }
        AstKind::Expressions => excute_exprs(ast, buf),
        AstKind::FuncionCall(_func, _type) => {
            execute_funccall(ast, buf);
            buf.output_pop("rax");
        }
        _ => unsupported_ast_err(&ast),
    }
}

pub fn output_ast<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::Operation(_) => output_operation_ast(ast, buf),
        AstKind::Control(_) => output_control_ast(ast, buf),
        AstKind::ImmidiateInterger(_num) => push_number(ast, buf),
        AstKind::Variable(_val) => push_variable_value(ast, buf),
        AstKind::Address => push_address(ast, buf),
        AstKind::Deref => push_deref_value(ast, buf),
        AstKind::Index => push_array_elem_value(ast, buf),
        AstKind::Expressions => excute_exprs(ast, buf),
        AstKind::FuncionCall(_func, _type) => execute_funccall(ast, buf),
        _ => unsupported_ast_err(&ast),
    }
}

fn output_push_arg_to_stack<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    push_variable_address(ast, buf);
    write_pop_two_values(buf);
    buf.output("    mov [rax], rdi");
}

// 引数をローカルスタックに格納する
pub fn output_push_args_to_stack<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
    if let Some(args_ast) = ast.exprs {
        let argcount = args_ast.len();
        // 後ろの引数からスタックに積んでいく
        for register in FUNC_ARG_REGISTERS[0..argcount].iter().rev() {
            buf.output_push(*register);
        }
        for arg_ast in args_ast {
            output_push_arg_to_stack(arg_ast, buf);
        }
    }
}

fn output_function<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    match &ast.kind {
        AstKind::FunctionImplementation((func_name, local_val_size)) => {
            output_function_prelude(func_name, local_val_size, buf);
            let func_context_ast = ast.context.take().unwrap();
            // 引数をスタックフレームに格納
            output_push_args_to_stack(ast, buf);
            output_ast(*func_context_ast, buf);
            output_function_epilogue(buf);
        }
        _ => unsupported_ast_err(&ast),
    }
}

fn write_assembly_header<T: Write>(buf: &mut OutputBuffer<T>) {
    buf.output(".intel_syntax noprefix");
    buf.output(".globl main");
}

pub fn output_assembly(asts: Vec<Ast>, output_file: &Path) {
    let buf = BufWriter::new(fs::File::create(output_file).unwrap());
    let mut outputbuf = OutputBuffer::new(buf);
    write_assembly_header(&mut outputbuf);
    for ast in asts {
        output_function(ast, &mut outputbuf);
    }
}
