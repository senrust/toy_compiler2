use crate::ast::ast::*;
use crate::definition::definitions::Definitions;
use crate::definition::number::Number;
use crate::definition::symbols::*;
use crate::definition::types::evaluate_binary_operation_type;
use crate::token::token::*;

// unary = primary |  + primary |  - primary | ! unary
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
        // drop "!" token
        let not_info = tokens.consume_symbol(Symbol::Not);
        let operand_ast = ast_unary(tokens, definitions);
        // とりあえず8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        Ast::new_single_operation_ast(Operation::Not, not_info, type_, operand_ast)
    } else {
        ast_primary(tokens, definitions)
    }
}

// mul = unary | (* unary | / unary)*
fn ast_mul(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_unary(tokens, definitions);
    let mut operation;
    let mut mul_ast = left_ast;
    loop {
        if tokens.expect_symbol(Symbol::Mul) {
            operation = Operation::Mul;
        } else if tokens.expect_symbol(Symbol::Div) {
            operation = Operation::Div;
        } else {
            return mul_ast;
        }

        let ast_info = tokens.consume();
        let right_ast = ast_unary(tokens, definitions);
        let type_ = evaluate_binary_operation_type(&mul_ast, &right_ast);
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
        let type_ = evaluate_binary_operation_type(&add_ast, &right_ast);
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

// assign = bit_operation ("=" assign)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
pub fn ast_assign(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let assignee_ast = ast_bit_operation(Symbol::BitOr, tokens, definitions);
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
