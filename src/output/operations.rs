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
    buf.output("    mov [rdi], rax");
    buf.output_push("rax");
}

fn exetute_mul<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::Operation(Operation::Mul) = ast.kind {
        output_ast(*ast.right.take().unwrap(), buf);
        output_ast(*ast.left.take().unwrap(), buf);
        write_operation(buf, "imul");
    } else {
        unexpected_ast_err(&ast, "operation *");
    }
}

fn exetute_div<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    if let AstKind::Operation(Operation::Div | Operation::Rem) = ast.kind {
        output_ast(*ast.right.take().unwrap(), buf);
        output_ast(*ast.left.take().unwrap(), buf);
        write_pop_two_values(buf);
        buf.output("    cqo");
        buf.output("    idiv rdi");
        if ast.kind == AstKind::Operation(Operation::Div) {
            buf.output_push("rax");
        } else {
            buf.output_push("rdx");
        }
    } else {
        unexpected_ast_err(&ast, "operation / or %");
    }
}

fn exetute_add<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let operation = match ast.kind {
        AstKind::Operation(Operation::Add) => "add",
        AstKind::Operation(Operation::Sub) => "sub",
        _ => unexpected_ast_err(&ast, "operation + or -"),
    };

    output_ast(*ast.right.take().unwrap(), buf);
    output_ast(*ast.left.take().unwrap(), buf);
    write_operation(buf, operation);
}

fn exetute_eq<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let euality = match ast.kind {
        AstKind::Operation(Operation::Eq) => "sete",
        AstKind::Operation(Operation::NotEq) => "setne",
        _ => unexpected_ast_err(&ast, "operation == or !="),
    };
    output_ast(*ast.right.take().unwrap(), buf);
    output_ast(*ast.left.take().unwrap(), buf);
    write_compararison(buf, euality);
}

fn exetute_comp<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
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
        _ => unexpected_ast_err(&ast, "operation >, <, >= or <="),
    };
    output_ast(*ast.right.take().unwrap(), buf);
    output_ast(*ast.left.take().unwrap(), buf);
    write_compararison(buf, comparison);
}

fn exetute_not<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    output_ast(*ast.operand.take().unwrap(), buf);
    write_value_compararison(buf, "sete", 0);
}

fn exetute_bitnot<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let instruction = "    not rax";
    output_ast(*ast.operand.take().unwrap(), buf);
    buf.output_pop("rax");
    buf.output(instruction);
    buf.output_push("rax");
}

fn exetute_assign<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    // 左辺値が被代入可能か確認
    let left_ast = ast.left.take().unwrap();
    match &left_ast.kind {
        AstKind::Variable(_val) => {
            push_variable_address(*left_ast, buf);
        }
        AstKind::Deref => {
            push_pointer_address(*left_ast, buf);
        }
        AstKind::Index => {
            push_array_elem_address(*left_ast, buf);
        }
        _ => unassignable_ast_err(&ast),
    }
    output_ast(*ast.right.take().unwrap(), buf);
    write_assignment(buf);
}

fn exetute_bit_operation<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    let bit_operation = match ast.kind {
        AstKind::Operation(Operation::BitAnd) => "and",
        AstKind::Operation(Operation::BitOr) => "or",
        AstKind::Operation(Operation::BitXor) => "xor",
        _ => unexpected_ast_err(&ast, "operation &, | or ^"),
    };

    output_ast(*ast.left.take().unwrap(), buf);
    output_ast(*ast.right.take().unwrap(), buf);
    write_operation(buf, bit_operation);
}

fn exetute_logical_and<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    // andのネストがある場合(A && B && C ), Aが0の時点でB, Cの比較をせずにネストを抜けることが可能.
    // これを行うにはAndのネスト判定とジャンプ先ラベルを保持している必要がある.
    // これを行うには状態管理が必要だが, 実装の手間が増えるので今回は逐次比較を行う
    let false_label_index = buf.label_index;
    let end_label_index = buf.label_index + 2;
    let false_label = format!("Label{}:", false_label_index);
    let end_label = format!("Label{}:", end_label_index);
    let jump_false = format!("    je Label{}", false_label_index);
    let jump_end = format!("    jmp Label{}", end_label_index);
    buf.label_index += 2;

    let comp_zero = "    cmp rax, 0";
    // 左側の値を計算
    output_ast(*ast.left.take().unwrap(), buf);
    buf.output_pop("rax");
    // 0と比較
    buf.output(comp_zero);
    // 0ならばFalse時の処理を行う
    buf.output(&jump_false);
    // 右側の値を計算
    output_ast(*ast.right.take().unwrap(), buf);
    buf.output_pop("rax");
    // 0と比較
    buf.output(comp_zero);
    // 0ならばFalse時の処理を行う
    buf.output(&jump_false);

    // True時の処理を記載
    buf.output_push_num(1);
    // Endへジャンプ
    buf.output(&jump_end);

    // Falseラベルを貼り, False時の処理を記載
    buf.output(&false_label);
    buf.output_push_num(0);
    buf.output(&end_label);
}

fn exetute_logical_or<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    // ORがネストされた状態((A || B || C)内のどれかで1になったらネストを抜けた先までジャンプできる
    // これにより高速化できるが, ジャンプ先管理の手間が増えるので今回はネストごとに逐次比較する
    let true_label_index = buf.label_index;
    let false_label_index = buf.label_index + 1;
    let end_label_index = buf.label_index + 2;
    let true_label = format!("Label{}:", true_label_index);
    let false_label = format!("Label{}:", false_label_index);
    let end_label = format!("Label{}:", end_label_index);
    let jump_true = format!("    jne Label{}", true_label_index);
    let jump_false = format!("    je Label{}", false_label_index);
    let jump_end = format!("    jmp Label{}", end_label_index);
    buf.label_index += 3;

    let comp_zero = "    cmp rax, 0";
    // 左側の値を計算
    output_ast(*ast.left.take().unwrap(), buf);
    buf.output_pop("rax");
    // 0と比較
    buf.output(comp_zero);
    // 1ならば(0でないならば)True時の処理を行う
    buf.output(&jump_true);
    // 右側の値を計算
    output_ast(*ast.right.take().unwrap(), buf);
    buf.output_pop("rax");
    // 0と比較
    buf.output(comp_zero);
    // 0ならばFalse時の処理に飛ぶ
    // 1のときはそのままtrueに行く
    buf.output(&jump_false);

    // True時の処理
    buf.output(&true_label);
    buf.output_push_num(1);
    // End文へ飛ぶ
    buf.output(&jump_end);

    // False時の処理
    buf.output(&false_label);
    buf.output_push_num(0);
    // Endラベルを挿入
    buf.output(&end_label);
}

fn exetute_increment<T: Write>(mut ast: Ast, buf: &mut OutputBuffer<T>) {
    // 前置インクリメント -> val = val + 1 を実行してスタックに積む
    // 後置インクリメント -> valをスタックに積み, val=val + 1を実行してスタックに積む. そしてスタックから削除
    if AstKind::Operation(Operation::ForwardIncrement) == ast.kind {
        output_ast(*ast.operand.take().unwrap(), buf);
    } else {
        output_ast(*ast.right.take().unwrap(), buf);
        output_ast(*ast.left.take().unwrap(), buf);
        buf.output_pop("rax");
    }
}

pub fn output_operation_ast<T: Write>(ast: Ast, buf: &mut OutputBuffer<T>) {
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
        AstKind::Operation(Operation::BitNot) => exetute_bitnot(ast, buf),
        AstKind::Operation(Operation::And) => exetute_logical_and(ast, buf),
        AstKind::Operation(Operation::Or) => exetute_logical_or(ast, buf),
        AstKind::Operation(Operation::ForwardIncrement | Operation::BackwardIncrement) => {
            exetute_increment(ast, buf)
        }
        _ => unsupported_ast_err(&ast),
    }
}
