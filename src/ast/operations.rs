use crate::ast::ast::*;
use crate::definition::definitions::Definitions;
use crate::definition::number::Number;
use crate::definition::symbols::Symbol;
use crate::definition::types::Type;
use crate::definition::reservedwords::Reserved;
use crate::token::error::*;
use crate::token::token::*;
use std::ops::Deref;

// ビット演算が可能なASTかチェックする
fn can_execute_bit_operation() {

}

// 算術演算が可能なASTかチェックする
fn can_execute_arithmetic_operation() {

}

// 論理演算が可能なASTかチェックする
fn can_execute_logical_operation() {

}

// 2引数の演算ではより大きな型に拡張して行う必要があるため,
// 型が異なる場合は型変換のASTを挟むようにする
pub fn expand_binary_operation_type(left: &Ast, _right: &Ast) -> Type {
    left.type_.clone()
}

// 代入では被代入側の型に合わせる必要がある
// 型が異なる場合は型変換のASTを挟むようにする
pub fn expand_assign_operation_type(left: &Ast, _right: &Ast) -> Type {
    left.type_.clone()
}

fn ast_not(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // drop "!" token
    let not_info = tokens.consume_symbol(Symbol::Not);
    let operand_ast = ast_unary(tokens, definitions);
    let type_ = operand_ast.type_.clone();
    Ast::new_single_operation_ast(Operation::Not, not_info, type_, operand_ast)
}

fn ast_bitnot(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // drop "~" token
    let bitnot_info = tokens.consume_symbol(Symbol::BitNot);
    let operand_ast = ast_unary(tokens, definitions);
    let type_ = operand_ast.type_.clone();
    Ast::new_single_operation_ast(Operation::BitNot, bitnot_info, type_, operand_ast)
}

fn ast_address(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // drop "&" token
    let address_info = tokens.consume_symbol(Symbol::BitAnd);
    let operand_ast = ast_unary(tokens, definitions);
    let type_;
    // 変数とプリミティブ型のみアドレスにすることができる
    if let AstKind::Variable(_) = &operand_ast.kind {
        type_ = Type::new_pointer(operand_ast.type_.clone());
    } else if operand_ast.type_.primitive.is_some() {
        type_ = Type::new_pointer(definitions.get_type("void").unwrap());
    } else {
        output_unaddressable_err(&address_info);
    }
    Ast::new_address_ast(address_info, type_, operand_ast)
}

fn ast_deref_pointer(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // drop "*" token
    let deref_info = tokens.consume_symbol(Symbol::Mul);
    let operand_ast = ast_unary(tokens, definitions);
    if let Some(deref_type) = &operand_ast.type_.pointer {
        let type_ = deref_type.deref().clone();
        Ast::new_deref_ast(deref_info, type_, operand_ast)
    } else {
        output_undereferensable_err(&deref_info);
    }
}

// sizeof = "sizeof" "(" formula ")"
fn ast_sizeof(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let sizeof_info = tokens.consume_reserved(Reserved::Sizeof);
    let size_ast = ast_formula(tokens, definitions);
    let size = size_ast.type_.size;
    let type_ = definitions.get_type("long").unwrap();
    let num = Number::U64(size as u64);
    Ast::new_integer_ast(num, sizeof_info, type_)
}

// unary = primary |  "+" unary |  "-" unary | "!" unary |  "~" unary | "&" unary |  "*" unary | "sizeof" "(" formula ")"
// この部分の規格は不明
pub fn ast_unary(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_symbol(Symbol::Add) {
        // drop "+" token
        tokens.consume_symbol(Symbol::Add);
        ast_primary(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::Sub) {
        // drop "-" token
        let sub_info = tokens.consume_symbol(Symbol::Sub);
        let primary_ast = ast_primary(tokens, definitions);
        let type_ = primary_ast.type_.clone();
        let zero_ast = Ast::new_integer_ast(Number::U64(0), sub_info, type_.clone());
        Ast::new_binary_operation_ast(Operation::Sub, sub_info, type_, zero_ast, primary_ast)
    } else if tokens.expect_symbol(Symbol::Not) {
        ast_not(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::BitNot) {
        ast_bitnot(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::BitAnd) {
        ast_address(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::Mul) {
        ast_deref_pointer(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::Sizeof) {
        ast_sizeof(tokens, definitions)
    } else {
        ast_primary(tokens, definitions)
    }
}

// mul = unary | (* unary | / unary　| % unary)*
fn ast_mul(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_unary(tokens, definitions);
    let mut operation;
    let mut mul_ast = left_ast;
    loop {
        if tokens.expect_symbol(Symbol::Mul) {
            operation = Operation::Mul;
        } else if tokens.expect_symbol(Symbol::Div) {
            operation = Operation::Div;
        } else if tokens.expect_symbol(Symbol::Rem) {
            operation = Operation::Rem;
        } else {
            return mul_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = ast_unary(tokens, definitions);
        let type_ = expand_binary_operation_type(&mul_ast, &right_ast);
        mul_ast = Ast::new_binary_operation_ast(operation, ast_info, type_, mul_ast, right_ast);
    }
}

// add = mul | (+  mul | - mul)*
fn ast_add(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_mul(tokens, definitions);
    let mut operation;
    let mut add_ast = left_ast;
    loop {
        if tokens.expect_symbol(Symbol::Add) {
            operation = Operation::Add;
        } else if tokens.expect_symbol(Symbol::Sub) {
            operation = Operation::Sub;
        } else {
            return add_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = ast_mul(tokens, definitions);
        let type_ = expand_binary_operation_type(&add_ast, &right_ast);
        add_ast = Ast::new_binary_operation_ast(operation, ast_info, type_, add_ast, right_ast);
    }
}

// relational = add (">" add | "<" add | ">=" add| "<=" add)*
fn ast_relational(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_add(tokens, definitions);
    let mut operation;
    let mut relational_ast = left_ast;
    loop {
        if tokens.expect_symbol(Symbol::Gt) {
            operation = Operation::Gt;
        } else if tokens.expect_symbol(Symbol::Lt) {
            operation = Operation::Lt;
        } else if tokens.expect_symbol(Symbol::Ge) {
            operation = Operation::Ge;
        } else if tokens.expect_symbol(Symbol::Le) {
            operation = Operation::Le;
        } else {
            return relational_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = ast_add(tokens, definitions);
        // とりあえず型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        relational_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, relational_ast, right_ast);
    }
}

// equality = relational ("==" relational | "!=" relational)*
pub fn ast_equality(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_relational(tokens, definitions);
    let mut operation;
    let mut equality_ast = left_ast;
    loop {
        if tokens.expect_symbol(Symbol::Eq) {
            operation = Operation::Eq;
        } else if tokens.expect_symbol(Symbol::NotEq) {
            operation = Operation::NotEq;
        } else {
            return equality_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = ast_add(tokens, definitions);
        // とりあえず和の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        equality_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, equality_ast, right_ast);
    }
}

// bit_operation = bit_or
// bit_or = bit_xor "|" bit_xor
// bit_xor = bit_and "^" bit_and
// bit_and = equality "&" equality
fn ast_bit_operation(
    bit_ope_symbol: Symbol,
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Ast {
    let (left_ast, bit_operation) = match &bit_ope_symbol {
        Symbol::BitOr => (
            ast_bit_operation(Symbol::BitXor, tokens, definitions),
            Operation::BitOr,
        ),
        Symbol::BitXor => (
            ast_bit_operation(Symbol::BitAnd, tokens, definitions),
            Operation::BitXor,
        ),
        Symbol::BitAnd => (ast_equality(tokens, definitions), Operation::BitAnd),
        _ => unreachable!(),
    };

    let mut bit_operation_ast = left_ast;
    loop {
        if !tokens.expect_symbol(bit_ope_symbol) {
            return bit_operation_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = match &bit_ope_symbol {
            Symbol::BitOr => ast_bit_operation(Symbol::BitXor, tokens, definitions),
            Symbol::BitXor => ast_bit_operation(Symbol::BitAnd, tokens, definitions),
            Symbol::BitAnd => ast_equality(tokens, definitions),
            _ => unreachable!(),
        };
        // とりあえずビット演算の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        bit_operation_ast = Ast::new_binary_operation_ast(
            bit_operation,
            ast_info,
            type_,
            bit_operation_ast,
            right_ast,
        );
    }
}

// logical = logical_or
// logical_or  = logical_and "||" logical_and
// logical_and = bit_operation "&&" bit_operation
fn ast_logical(logical_symbol: Symbol, tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (left_ast, logical_op) = match &logical_symbol {
        Symbol::Or => (ast_logical(Symbol::And, tokens, definitions), Operation::Or),
        Symbol::And => (
            ast_bit_operation(Symbol::BitOr, tokens, definitions),
            Operation::And,
        ),
        _ => unreachable!(),
    };

    let mut logical_op_ast = left_ast;
    loop {
        if !tokens.expect_symbol(logical_symbol) {
            return logical_op_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = match &logical_symbol {
            Symbol::Or => ast_logical(Symbol::And, tokens, definitions),
            Symbol::And => ast_bit_operation(Symbol::BitOr, tokens, definitions),
            _ => unreachable!(),
        };
        // とりあえずビット演算の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        logical_op_ast =
            Ast::new_binary_operation_ast(logical_op, ast_info, type_, logical_op_ast, right_ast);
    }
}

// formula = logical
pub fn ast_formula(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    ast_logical(Symbol::Or, tokens, definitions)
}

// assign = formula ("=" assign)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
pub fn ast_assign(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let assignee_ast = ast_formula(tokens, definitions);
    let mut assign_ast = assignee_ast;
    loop {
        if !tokens.expect_symbol(Symbol::Assign) {
            return assign_ast;
        }
        let ast_info = tokens.consume_symbol(Symbol::Assign);
        let ast_assigner = ast_assign(tokens, definitions);
        // とりあえず代入の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        assign_ast = Ast::new_binary_operation_ast(
            Operation::Assign,
            ast_info,
            type_,
            assign_ast,
            ast_assigner,
        );
    }
}