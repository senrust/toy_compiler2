use std::io::Write;

use crate::ast::ast::*;
use crate::ast::error::*;
use crate::output::output::*;

macro_rules! func_arg_register {
    () => {
        ["rdi", "rdx", "rcx", "r8", "r9"]
    };
}

// return文のコンパイル
// returnする値のastはexprs[0]
pub fn execute_return<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let mut expr = ast.exprs.take().unwrap();
    let return_value = expr.first_mut().unwrap();
    output_ast(return_value, buf);
    buf.output_pop("rax");
    output_function_epilogue(buf);
}

// if文のコンパイル
// if文の条件はcontext, true時の条件はother[0], elseがある場合はelse時の条件はother[1]にある
pub fn execute_if<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    let label_index = buf.get_label_index();

    // 条件式のコンパイル
    let mut condition = ast.context.take().unwrap();
    let mut if_context = ast.other.take().unwrap();
    let has_else = if_context[1].is_some();
    output_ast(&mut condition, buf);
    buf.output_pop("rax");
    buf.output("    cmp rax, 0");
    if has_else {
        buf.output(&format!("    je .LabelElse{}", label_index));
    } else {
        buf.output(&format!("    je .LabelIfEnd{}", label_index));
    }

    output_expr_ast(if_context[0].as_mut().unwrap(), buf);
    // else文がある場合
    if has_else {
        buf.output(&format!(".LabelElse{}:", label_index));
        output_expr_ast(&mut if_context[1].as_mut().unwrap(), buf);
    } else {
        buf.output(&format!(".LabelIfEnd{}:", label_index));
    }
    buf.increment_label();
}

// for文のコンパイル
// for文はexprs[0]に初期化式, exprs[1]に条件式, exprs[2]に更新式がある
pub fn execute_for<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    // ループ情報の作成
    buf.enter_loop_control(LoopKind::For);
    let label_index = buf.get_label_index();
    let mut for_conditions = ast.other.take().unwrap();
    let mut for_context = ast.context.take().unwrap();
    // 初期化式
    if let Some(ref mut initialize_ast) = for_conditions[0] {
        output_expr_ast(initialize_ast, buf);
    }
    // ループ開始ラベル
    buf.output(&format!(".LabelForBegin{}:", label_index));
    // 条件式
    if let Some(ref mut condition_ast) = for_conditions[1] {
        output_expr_ast(condition_ast, buf);
        // 条件式が成立する場合はif文のEndまでジャンプ
        buf.output("    cmp rax, 1");
        buf.output(&format!("    je .LabelForEnd{}", label_index));
    }
    // for内容
    output_expr_ast(&mut for_context, buf);
    // 更新式
    if let Some(ref mut condition_ast) = for_conditions[2] {
        output_expr_ast(condition_ast, buf);
    }
    buf.output(&format!("    jmp .LabelForBegin{}", label_index));
    buf.output(&format!(".LabelForEnd{}:", label_index));
    // ループ情報の削除
    buf.exit_loop_control();
    buf.increment_label();
}

// while文のコンパイル
// while文はcontextに条件式,
// expr[0]にwhile内容がある
pub fn execute_while<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    // ループ情報の作成
    buf.enter_loop_control(LoopKind::While);
    let label_index = buf.get_label_index();

    let mut while_condition = ast.context.take().unwrap();
    let while_context = &mut ast.exprs.take().unwrap()[0];

    buf.output(&format!(".LabelWhileBegin{}:", label_index));
    // 条件式
    output_ast(&mut while_condition, buf);
    // 条件式が成立しない場合はWhile文のEndまでジャンプ
    buf.output("    cmp rax, 0");
    buf.output(&format!("    je .LabelWhileEnd{}", label_index));
    // while内容
    output_expr_ast(while_context, buf);
    buf.output(&format!("    jmp .LabelWhileBegin{}", label_index));
    buf.output(&format!(".LabelWhileEnd{}:", label_index));
    // ループ情報の削除
    buf.exit_loop_control();
    buf.increment_label();
}

// break文のコンパイル
pub fn execute_break<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    if let Ok(break_dist_label) = buf.get_break_label() {
        buf.output(&break_dist_label);
    } else {
        invalid_direction_err(ast, "break");
    }
}

pub fn execute_funccall<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::FuncionCall(fucname) = &ast.kind {
        let functype = ast.type_.function.take().unwrap();
        // push args
        if let Some(args_ast) = &mut ast.exprs {
            let arg_count = args_ast.len();
            for arg_ast in args_ast {
                output_ast(arg_ast, buf);
            }
            // set args in register
            for i in 0..arg_count {
                let registers = func_arg_register!();
                buf.output_pop(registers[i]);
            }
        }
        buf.output(&format!("    call {}", fucname));
        // push ret
        if let Some(_ret_type) = &functype.ret {
            buf.output_push("rax");
        }
    } else {
        invalid_direction_err(ast, "call function");
    }
}
