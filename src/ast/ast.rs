use crate::definition::definitions::Definitions;
use crate::definition::functions::Function;
use crate::definition::number::Number;
use crate::definition::reservedwords::*;
use crate::definition::symbols::*;
use crate::definition::types::{evaluate_binary_operation_type, Type};
use crate::definition::variables::*;
use crate::token::error::*;
use crate::token::token::{TokenInfo, Tokens};

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Eq,     // ==
    NotEq,  // !=
    Gt,     // >
    Lt,     // <
    Ge,     // >=
    Le,     // <=
    Not,    // !
    Assign, // =
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Return,
    If,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstKind {
    FunctionImplementation((String, usize)),
    FuncionCall(String),
    Expressions,
    Operation(Operation),
    Control(Control),
    Variable(Variable),
    ImmidiateInterger(Number),
}

#[derive(Debug)]
pub struct Ast {
    pub kind: AstKind,
    pub info: TokenInfo,
    pub type_: Type,
    pub left: Option<Box<Ast>>,
    pub right: Option<Box<Ast>>,
    pub operand: Option<Box<Ast>>,
    pub exprs: Option<Vec<Ast>>,
    pub context: Option<Box<Ast>>,
}

impl Ast {
    fn new_integer_ast(num: Number, info: TokenInfo, type_: Type) -> Ast {
        Ast {
            kind: AstKind::ImmidiateInterger(num),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: None,
            context: None,
        }
    }

    fn new_variable_ast(val: Variable, info: TokenInfo, type_: Type) -> Ast {
        Ast {
            kind: AstKind::Variable(val),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: None,
            context: None,
        }
    }

    fn new_single_operation_ast(
        operation: Operation,
        info: TokenInfo,
        type_: Type,
        operand: Ast,
    ) -> Ast {
        Ast {
            kind: AstKind::Operation(operation),
            info,
            type_,
            left: None,
            right: None,
            operand: Some(Box::new(operand)),
            exprs: None,
            context: None,
        }
    }

    fn new_binary_operation_ast(
        operation: Operation,
        info: TokenInfo,
        type_: Type,
        left: Ast,
        right: Ast,
    ) -> Ast {
        Ast {
            kind: AstKind::Operation(operation),
            info,
            type_,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            operand: None,
            exprs: None,
            context: None,
        }
    }

    fn new_function_implementation_ast(
        func_name: &str,
        info: TokenInfo,
        type_: Type,
        frame_size: usize,
        context: Ast,
    ) -> Ast {
        Ast {
            kind: AstKind::FunctionImplementation((func_name.to_string(), frame_size)),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: None,
            context: Some(Box::new(context)),
        }
    }

    fn new_expressions_ast(
        info: TokenInfo,
        type_: Type,
        exprs: Vec<Ast>,
        context: Option<Box<Ast>>,
    ) -> Ast {
        Ast {
            kind: AstKind::Expressions,
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: Some(exprs),
            context,
        }
    }

    fn new_control_ast(
        info: TokenInfo,
        type_: Type,
        control: Control,
        context: Option<Box<Ast>>,
        exprs: Vec<Ast>,
    ) -> Ast {
        Ast {
            kind: AstKind::Control(control),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: Some(exprs),
            context,
        }
    }
}

fn ast_number(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_number() {
        output_unexpected_token_err(tokens);
    }

    if let Ok((num, info)) = tokens.consume_integer() {
        let type_ = definitions.get_primitive_type(&num);
        match num {
            Number::U64(num_u64) => Ast::new_integer_ast(Number::U64(num_u64), info, type_),
            Number::F64(_num_f64) => unreachable!(),
        }
    } else {
        invalid_number_token_err(&tokens.get().unwrap().info);
    }
}

fn ast_variable(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_identifier() {
        output_unexpected_token_err(tokens);
    }

    if let Ok((ident, info)) = tokens.consume_identifier() {
        let val;
        if let Some(defined_val) = definitions.get_variable(&ident) {
            val = defined_val;
        } else {
            // とりあえず8バイトのlong型とする
            let type_ = definitions.get_type("long").unwrap();
            val = definitions.declear_local_val(&ident, type_).unwrap();
        }
        let val_type = val.get_type();
        Ast::new_variable_ast(val, info, val_type)
    } else {
        output_unexpected_token_err(tokens);
    }
}

// primary = num | variable | "(" add ")"
fn ast_primary(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_number() {
        ast_number(tokens, definitions)
    } else if tokens.expect_identifier() {
        ast_variable(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::LeftParenthesis) {
        // drop "(" token
        tokens.consume().unwrap();
        let add_ast = ast_equality(tokens, definitions);
        if tokens.expect_symbol(Symbol::RightParenthesis) {
            // drop ")" token
            tokens.consume().unwrap();
            add_ast
        } else {
            output_unclosed_token_err(tokens);
        }
    } else {
        output_unexpected_token_err(tokens);
    }
}

// unary = primary |  + primary |  - primary | ! unary
fn ast_unary(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_symbol(Symbol::Add) {
        // drop "+" token
        tokens.consume().unwrap();
        ast_primary(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::Sub) {
        // drop "-" token
        let sub_info = tokens.consume().unwrap();
        let primary_ast = ast_primary(tokens, definitions);
        let type_ = primary_ast.type_.clone();
        let zero_ast = Ast::new_integer_ast(Number::U64(0), sub_info.clone(), type_.clone());
        Ast::new_binary_operation_ast(Operation::Sub, sub_info, type_, zero_ast, primary_ast)
    } else if tokens.expect_symbol(Symbol::Not) {
        // drop "!" token
        let not_info = tokens.consume().unwrap();
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

        let ast_info = tokens.consume().unwrap();
        let right_ast = ast_unary(tokens, definitions);
        let type_ = evaluate_binary_operation_type(&mul_ast, &right_ast).unwrap();
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

        let ast_info = tokens.consume().unwrap();
        let right_ast = ast_mul(tokens, definitions);
        let type_ = evaluate_binary_operation_type(&add_ast, &right_ast).unwrap();
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

        let ast_info = tokens.consume().unwrap();
        let right_ast = ast_add(tokens, definitions);
        // とりあえず比較の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        relational_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, relational_ast, right_ast);
    }
}

// equality = relational ("==" relational | "!=" relational)*
fn ast_equality(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
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

        let ast_info = tokens.consume().unwrap();
        let right_ast = ast_add(tokens, definitions);
        // とりあえず比較の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        equality_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, equality_ast, right_ast);
    }
}

// assign = equality ("=" equality)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
fn ast_assign(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let assignee_ast = ast_equality(tokens, definitions);
    let mut assign_ast = assignee_ast;
    loop {
        if !tokens.expect_symbol(Symbol::Assign) {
            return assign_ast;
        }
        let ast_info = tokens.consume().unwrap();
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

// return = "return" assign
// return は returnする対象をもつ
fn ast_return(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_reserved(Reserved::Return) {
        output_unexpected_token_err(tokens);
    }
    // consume "return"
    let info = tokens.consume().unwrap();
    let return_value = ast_assign(tokens, definitions);
    let type_ = return_value.type_.clone();
    // 今後関数の定義されている戻り型と比較を行う
    // 即;ならばvoid型に設定する
    let context = vec![return_value];
    Ast::new_control_ast(info, type_, Control::Return, None, context)
}

// if = "if" "(" assign ")" expr ("else" expr)?
// if は contextに条件式, exprs[0]に trueのAst, exprs[1]にfalseのAstが入る
fn ast_if(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_reserved(Reserved::If) {
        output_unexpected_token_err(tokens);
    }

    let mut if_ast_vec: Vec<Ast> = vec![];
    // consume "if"
    let if_info = tokens.consume().unwrap();
    let if_type = definitions.get_type("void").unwrap();
    if !tokens.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // consume "("
    tokens.consume().unwrap();
    let condition_ast = ast_assign(tokens, definitions);
    if !tokens.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // consume ")"
    tokens.consume().unwrap();
    // true時のAst
    let true_ast = ast_expr(tokens, definitions);
    if_ast_vec.push(true_ast);
    if tokens.expect_reserved(Reserved::Else) {
        // consume "else"
        tokens.consume().unwrap();
        let else_ast = ast_expr(tokens, definitions);
        if_ast_vec.push(else_ast);
    }
    Ast::new_control_ast(
        if_info,
        if_type,
        Control::If,
        Some(Box::new(condition_ast)),
        if_ast_vec,
    )
}

// expr = exprs  |
//        "return" assign
//        "if" "(" assign ")" expr ("else" expr)?
//        assign |
fn ast_expr(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_symbol(Symbol::LeftCurlyBracket) {
        ast_exprs(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::Return) {
        ast_return(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::If) {
        ast_if(tokens, definitions)
    } else {
        ast_assign(tokens, definitions)
    }
}

// exprs = "{" (expr ";") *  + (expr)? "}"
fn ast_exprs(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let mut exprs: Vec<Ast> = vec![];

    if !tokens.expect_symbol(Symbol::LeftCurlyBracket) {
        output_unexpected_token_err(tokens);
    }

    // consume "{"
    tokens.consume().unwrap();
    // ローカル変数のネストを深くする
    definitions.enter_new_local_scope();

    while !tokens.expect_symbol(Symbol::RightCurlyBracket) {
        if tokens.is_empty() {
            output_unclosed_token_err(tokens);
        }
        let expr = ast_expr(tokens, definitions);
        if tokens.expect_symbol(Symbol::SemiColon) {
            // consume ";"
            tokens.consume().unwrap();
            exprs.push(expr);
        } else {
            // 複文, if文はセミコロン不要
            match &expr.kind {
                AstKind::Expressions => exprs.push(expr),
                AstKind::Control(Control::If) => exprs.push(expr),
                _ => output_unexpected_token_err(tokens),
            }
        }
    }
    // ローカル変数のスコープを抜ける
    definitions.exit_current_local_scope();

    // "}" の位置を複文の情報とする
    let exprs_info = tokens.consume().unwrap();
    let exprs_type; // 複文が返す型情報
    exprs_type = definitions.get_type("void").unwrap();
    Ast::new_expressions_ast(exprs_info, exprs_type, exprs, None)
}

// 関数の引数を取得します
fn get_func_args(tokens: &mut Tokens, _definitions: &mut Definitions) -> Option<Vec<Type>> {
    if !tokens.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // drop "("
    tokens.consume().unwrap();

    if !tokens.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // drop ")"
    tokens.consume().unwrap();
    None
}

fn ast_funcution_implementaion(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_identifier() {
        output_unexpected_token_err(tokens);
    }
    // 関数定義
    let (func_name, info) = tokens.consume_identifier().unwrap();
    let func_args = get_func_args(tokens, definitions);
    let func = Function::new(func_args, None);
    let func_type = definitions.declear_function(&func_name, func).unwrap();
    // 関数実装ASTを作成
    definitions.initialize_local_scope();
    let expfunc_context_ast = ast_exprs(tokens, definitions);
    let frame_size = definitions.get_local_val_frame_size();
    // 関数AST作成
    Ast::new_function_implementation_ast(
        &func_name,
        info,
        func_type,
        frame_size,
        expfunc_context_ast,
    )
}

pub fn make_asts(mut tokens: Tokens) -> Vec<Ast> {
    let mut asts: Vec<Ast> = vec![];
    let mut programinfo = Definitions::new();
    while tokens.has_token() {
        let ast = ast_funcution_implementaion(&mut tokens, &mut programinfo);
        asts.push(ast);
    }
    asts
}
