use std::io::Write;

use crate::ast::ast::*;
use crate::ast::error::*;
use crate::output::output::*;

// return文のコンパイル
// returnする値のastはexprs[0]
pub fn execute_return<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Control(Control::Return) => (),
        _ => unexpected_ast_err(ast, "return"),
    }
    let mut expr = ast.exprs.take().unwrap();
    let return_value = expr.first_mut().unwrap();
    output_ast(return_value, buf);
    buf.output_pop("rax");
    output_function_epilogue(buf);
}

// if文のコンパイル
// if文の条件はcontext, true時の条件はother[0], elseがある場合はelse時の条件はother[1]にある
pub fn execute_if<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Control(Control::If) => (),
        _ => unexpected_ast_err(ast, "if"),
    }

    // 条件式のコンパイル
    let mut condition = ast.context.take().unwrap();
    let mut if_context = ast.other.take().unwrap();
    let has_else = if_context[1].is_some();
    output_ast(&mut condition, buf);
    buf.output_pop("rax");
    buf.output("    cmp rax, 0");
    if has_else {
        buf.output(&format!("    je .LabelElse{}", buf.label_index));
    } else {
        buf.output(&format!("    je .LabelEnd{}", buf.label_index));
    }

    output_expr_ast(if_context[0].as_mut().unwrap(), buf);
    // else文がある場合
    if has_else {
        buf.output(&format!(".LabelElse{}:", buf.label_index));
        output_expr_ast(&mut if_context[1].as_mut().unwrap(), buf);
    } else {
        buf.output(&format!(".LabelEnd{}:", buf.label_index));
    }
    buf.increment_label();
}

// for文のコンパイル
// for文はexprs[0]に初期化式, exprs[1]に条件式, exprs[2]に更新式がある
pub fn execute_for<T: Write>(ast: &mut Ast, buf: &mut OutputBuffer<T>) {
    match ast.kind {
        AstKind::Control(Control::For) => (),
        _ => unexpected_ast_err(ast, "for"),
    }

    let mut for_conditions = ast.other.take().unwrap();
    let mut for_context = ast.context.take().unwrap();
    // 初期化式
    if let Some(ref mut initialize_ast) = for_conditions[0] {
        output_expr_ast(initialize_ast, buf);
    }
    // ループ開始ラベル
    buf.output(&format!(".LabelForBegin{}:", buf.label_index));
    // 条件式
    if let Some(ref mut condition_ast) = for_conditions[1] {
        output_expr_ast(condition_ast, buf);
        // 条件式が成立する場合はif文のEndまでジャンプ
        buf.output("    cmp rax, 1");
        buf.output(&format!("    je .LabelForend{}", buf.label_index));
    }
    // for内容
    output_expr_ast(&mut for_context, buf);
    // 更新式
    if let Some(ref mut condition_ast) = for_conditions[2] {
        output_expr_ast(condition_ast, buf);
    }
    buf.output(&format!("    jmp .LabelForBegin{}", buf.label_index));
    buf.output(&format!(".LabelForend{}:", buf.label_index));
    buf.increment_label();
}
