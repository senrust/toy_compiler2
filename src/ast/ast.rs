use crate::ast::controls::*;
use crate::ast::declaration::*;
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
    For,
    While,
    Break,
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
    pub other: Option<Vec<Option<Ast>>>,
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
            other: None,
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
            other: None,
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
            other: None,
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
            other: None,
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
            other: None,
        }
    }

    pub fn new_functioncall_ast(
        func_name: &str,
        info: TokenInfo,
        type_: Type,
        args: Option<Vec<Ast>>,
    ) -> Ast {
        Ast {
            kind: AstKind::FuncionCall(func_name.to_string()),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: args,
            context: None,
            other: None,
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
            other: None,
        }
    }

    pub fn new_control_ast(
        info: TokenInfo,
        type_: Type,
        control: Control,
        context: Option<Box<Ast>>,
        exprs: Option<Vec<Ast>>,
        other: Option<Vec<Option<Ast>>>,
    ) -> Ast {
        Ast {
            kind: AstKind::Control(control),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs,
            context,
            other,
        }
    }
}

fn ast_number(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (num, info) = tokens.consume_integer();
    let type_ = definitions.get_number_type(&num);
    match num {
        Number::U64(num_u64) => Ast::new_integer_ast(Number::U64(num_u64), info, type_),
        Number::F64(_num_f64) => unreachable!(),
    }
}

fn ast_variable(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (ident, info) = tokens.consume_identifier();
    if let Some(val) = definitions.get_variable(&ident) {
        let val_type = val.get_type();
        Ast::new_variable_ast(val, info, val_type)
    } else {
        output_undeclared_variable_err(&info);
    }
}

// primary = num | variable | functioncall | "(" add ")"
fn ast_primary(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_number() {
        ast_number(tokens, definitions)
    } else if tokens.expect_identifier() {
        if tokens.expect_next_symbol(Symbol::LeftParenthesis, 1) {
            ast_functioncall(tokens, definitions)
        } else {
            ast_variable(tokens, definitions)
        }
    } else if tokens.expect_symbol(Symbol::LeftParenthesis) {
        // drop "(" token
        tokens.consume_symbol(Symbol::LeftParenthesis);
        let add_ast = ast_equality(tokens, definitions);
        if tokens.expect_symbol(Symbol::RightParenthesis) {
            // drop ")" token
            tokens.consume_symbol(Symbol::RightParenthesis);
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
        tokens.consume_symbol(Symbol::Add);
        ast_primary(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::Sub) {
        // drop "-" token
        let sub_info = tokens.consume_symbol(Symbol::Sub);
        let primary_ast = ast_primary(tokens, definitions);
        let type_ = primary_ast.type_.clone();
        let zero_ast = Ast::new_integer_ast(Number::U64(0), sub_info.clone(), type_.clone());
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

        let ast_info = tokens.consume();
        let right_ast = ast_add(tokens, definitions);
        // とりあえず比較の型は8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        equality_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, equality_ast, right_ast);
    }
}

// assign = equality ("=" equality)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
pub fn ast_assign(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let assignee_ast = ast_equality(tokens, definitions);
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

// expr = exprs  |
//        "return" assign
//        "if"  "(" assign ")" expr ("else" expr)?
//        "for" "(" expr? ";" expr? ";" expr? ")" stmt
//        "while"  "(" assign ")" expr
//        "break"
//        assign |(";"を要求しないので注意)
pub fn ast_expr(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_symbol(Symbol::LeftCurlyBracket) {
        ast_exprs(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::Return) {
        ast_return(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::If) {
        ast_if(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::For) {
        ast_for(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::While) {
        ast_while(tokens, definitions)
    } else if tokens.expect_reserved(Reserved::Break) {
        ast_break(tokens, definitions)
    } else {
        ast_assign(tokens, definitions)
    }
}

// exprs = "{" (( expr   | type valname ) ";")* "}"
fn ast_exprs(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let mut exprs: Vec<Ast> = vec![];
    // comsume "{"
    tokens.consume_symbol(Symbol::LeftCurlyBracket);
    // ローカル変数のネストを深くする
    definitions.enter_new_local_scope();

    while !tokens.expect_symbol(Symbol::RightCurlyBracket) {
        if tokens.is_empty() {
            output_unclosed_token_err(tokens);
        }

        //ローカル変数宣言
        if is_type_token(tokens, definitions) {
            local_val_declaration(tokens, definitions);
            continue;
        }

        let expr = ast_expr(tokens, definitions);
        if tokens.expect_symbol(Symbol::SemiColon) {
            // consume ";"
            tokens.consume_symbol(Symbol::SemiColon);
            exprs.push(expr);
        } else {
            // 複文, if, for, while文はセミコロン不要
            match &expr.kind {
                AstKind::Expressions => exprs.push(expr),
                AstKind::Control(Control::If) => exprs.push(expr),
                AstKind::Control(Control::For) => exprs.push(expr),
                AstKind::Control(Control::While) => exprs.push(expr),
                _ => output_unexpected_token_err(tokens),
            }
        }
    }
    // ローカル変数のスコープを抜ける
    definitions.exit_current_local_scope();

    // "}" の位置を複文の情報とする
    let exprs_info = tokens.consume_symbol(Symbol::RightCurlyBracket);
    let exprs_type; // 複文が返す型情報
    exprs_type = definitions.get_type("void").unwrap();
    Ast::new_expressions_ast(exprs_info, exprs_type, exprs, None)
}

// 関数の引数を取得します
fn get_func_args(tokens: &mut Tokens, _definitions: &mut Definitions) -> Option<Vec<Type>> {
    // consume "("
    tokens.consume_symbol(Symbol::LeftParenthesis);
    // consume ")"
    tokens.consume_symbol(Symbol::RightParenthesis);
    None
}

fn ast_funcution_implementaion(
    func_name: String,
    func_info: TokenInfo,
    func_type: Type,
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Ast {
    // 関数実装ASTを作成
    definitions.initialize_local_scope();
    let expfunc_context_ast = ast_exprs(tokens, definitions);
    let frame_size = definitions.get_local_val_frame_size();
    definitions.clear_local_val_scope();
    // 関数AST作成
    Ast::new_function_implementation_ast(
        &func_name,
        func_info,
        func_type,
        frame_size,
        expfunc_context_ast,
    )
}

fn ast_function(
    func_name: String,
    func_info: TokenInfo,
    ret_type: Type,
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Option<Ast> {
    let func_args = get_func_args(tokens, definitions);
    let func_ret;
    if ret_type == definitions.get_type("void").unwrap() {
        func_ret = None;
    } else {
        func_ret = Some(ret_type);
    }
    let func = Function::new(func_args, func_ret);
    if let Ok(func_type) = definitions.declar_function(&func_name, func) {
        if tokens.expect_symbol(Symbol::SemiColon) {
            tokens.consume_symbol(Symbol::SemiColon);
            return None;
        } else if tokens.expect_symbol(Symbol::LeftCurlyBracket) {
            Some(ast_funcution_implementaion(
                func_name,
                func_info,
                func_type,
                tokens,
                definitions,
            ))
        } else {
            output_unexpected_token_err(tokens);
        }
    } else {
        output_unexpected_token_err(tokens);
    }
}

// グローバル変数定義, 関数宣言, 関数実装を行う
fn ast_global(tokens: &mut Tokens, definitions: &mut Definitions) -> Option<Ast> {
    let type_ = cousume_type_token(tokens, definitions);
    let (name, info) = tokens.consume_identifier();
    if tokens.expect_symbol(Symbol::LeftParenthesis) {
        ast_function(name, info, type_, tokens, definitions)
    } else {
        output_unexpected_token_err(tokens);
    }
}

pub fn make_asts(mut tokens: Tokens) -> Vec<Ast> {
    let mut asts: Vec<Ast> = vec![];
    let mut definitions = Definitions::new();
    while tokens.has_token() {
        if let Some(func_ast) = ast_global(&mut tokens, &mut definitions) {
            asts.push(func_ast);
        }
    }
    asts
}
