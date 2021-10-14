use std::ops::Deref;

use crate::ast::controls::*;
use crate::ast::declaration::*;
use crate::ast::operations::*;
use crate::definition::definitions::Definitions;
use crate::definition::functions::Function;
use crate::definition::number::Number;
use crate::definition::reservedwords::*;
use crate::definition::symbols::*;
use crate::definition::types::*;
use crate::definition::variables::*;
use crate::token::error::*;
use crate::token::token::TokenKind;
use crate::token::token::{TokenInfo, Tokens};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,     // ==
    NotEq,  // !=
    Gt,     // >
    Lt,     // <
    Ge,     // >=
    Le,     // <=
    Not,    // !
    Assign, // =
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    BitNot, // ~
    And,    // &&
    Or,     // ||
    ForwardIncrement,
    BackwardIncrement,
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
    FuncionCall(String, Type),
    Expressions,
    Operation(Operation),
    Control(Control),
    Variable(Variable),
    Address,
    Deref,
    Index,
    ImmidiateInterger(Number),
}

#[derive(Debug, Clone)]
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
    pub fn new_integer_ast(num: Number, info: TokenInfo, type_: Type) -> Ast {
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

    pub fn new_variable_ast(val: Variable, info: TokenInfo, type_: Type) -> Ast {
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

    pub fn new_address_ast(info: TokenInfo, type_: Type, operand: Ast) -> Ast {
        Ast {
            kind: AstKind::Address,
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

    pub fn new_deref_ast(info: TokenInfo, type_: Type, operand: Ast) -> Ast {
        Ast {
            kind: AstKind::Deref,
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

    pub fn new_index_ast(info: TokenInfo, type_: Type, val: Ast, index: Ast) -> Ast {
        Ast {
            kind: AstKind::Index,
            info,
            type_,
            left: Some(Box::new(val)),
            right: Some(Box::new(index)),
            operand: None,
            exprs: None,
            context: None,
            other: None,
        }
    }

    pub fn new_single_operation_ast(
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

    pub fn new_binary_operation_ast(
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

    pub fn new_function_implementation_ast(
        func_name: &str,
        info: TokenInfo,
        type_: Type,
        frame_size: usize,
        args_expr: Option<Vec<Ast>>,
        context: Ast,
    ) -> Ast {
        Ast {
            kind: AstKind::FunctionImplementation((func_name.to_string(), frame_size)),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: args_expr,
            context: Some(Box::new(context)),
            other: None,
        }
    }

    pub fn new_functioncall_ast(
        func_name: &str,
        info: TokenInfo,
        functype: Type,
        restype_: Type,
        args: Option<Vec<Ast>>,
    ) -> Ast {
        Ast {
            kind: AstKind::FuncionCall(func_name.to_string(), functype),
            info,
            type_: restype_,
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

pub fn ast_number(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (num, info) = tokens.consume_integer();
    let type_ = definitions.get_number_type(&num);
    match num {
        Number::U64(num_u64) => Ast::new_integer_ast(Number::U64(num_u64), info, type_),
        Number::F64(_num_f64) => unreachable!(),
    }
}

pub fn ast_variable(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (ident, info) = tokens.consume_identifier();
    if let Some(val) = definitions.get_variable(&ident) {
        let val_type = val.get_type();
        Ast::new_variable_ast(val, info, val_type)
    } else {
        output_undeclared_variable_err(&info);
    }
}

fn ast_index(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    tokens.consume_symbol(Symbol::LeftSquareBracket);
    let index_ast = ast_formula(tokens, definitions);
    if index_ast.type_.is_integer_type() {
        // ここでlong型の値にする必要がある
        tokens.consume_symbol(Symbol::RightSquareBracket);
        index_ast
    } else {
        output_unindexiable_err(&index_ast.info)
    }
}

// 配列アクセス
pub fn ast_array_access(val_ast: Ast, tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if val_ast.type_.is_array() {
        let (_array_len, index_type) = val_ast.type_.array.as_ref().unwrap();
        let index_type = index_type.deref().clone();
        let index_ast = ast_index(tokens, definitions);
        let mut array_access_ast = Ast::new_index_ast(val_ast.info, index_type, val_ast, index_ast);
        // 2次元配列のindexingをできるようにする
        while tokens.expect_symbol(Symbol::LeftSquareBracket) {
            let (_array_len, index_type) = array_access_ast.type_.array.as_ref().unwrap();
            let index_type = index_type.deref().clone();
            let index_ast = ast_index(tokens, definitions);
            array_access_ast = Ast::new_index_ast(
                array_access_ast.info,
                index_type,
                array_access_ast,
                index_ast,
            );
        }
        array_access_ast
    } else {
        output_unindexiable_err(&val_ast.info);
    }
}

// variable_op = variable | ( "."indet | -> indet | "[" formula "]")* | ("++" | "--" | "(" ")" )
// val->val.val[10].val++ (primaryである必要)
// val->val.val[10].val() (funcpointerである必要)
// に対応できるようにする
pub fn ast_variable_op(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let mut val_ast = ast_variable(tokens, definitions);
    loop {
        if tokens.expect_symbol(Symbol::LeftSquareBracket) {
            val_ast = ast_array_access(val_ast, tokens, definitions)
        } else if tokens.expect_symbols(&[Symbol::Increment, Symbol::Decrement]) {
            val_ast = ast_backward_increment(val_ast, tokens, definitions)
        } else {
            break;
        }
    }
    val_ast
}

// primary_op =  variable | functioncall
fn ast_primary_op(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if let TokenKind::Identifier(ident) = &tokens.get().unwrap().kind {
        if let Some(_func) = definitions.get_function(ident) {
            return ast_functioncall(tokens, definitions);
        } else {
            return ast_variable_op(tokens, definitions);
        }
    } else {
        unreachable!();
    }
}

// primary = num | primary_op | "(" formula ")" | | ("++" | "--") variable
pub fn ast_primary(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if tokens.expect_number() {
        ast_number(tokens, definitions)
    } else if tokens.expect_identifier() {
        ast_primary_op(tokens, definitions)
    } else if tokens.expect_symbol(Symbol::LeftParenthesis) {
        // drop "(" token
        tokens.consume_symbol(Symbol::LeftParenthesis);
        let formula_ast = ast_formula(tokens, definitions);
        tokens.consume_symbol(Symbol::RightParenthesis);
        formula_ast
    } else if tokens.expect_symbols(&[Symbol::Increment, Symbol::Decrement]) {
        ast_forward_increment(tokens, definitions)
    } else {
        output_unexpected_token_err(tokens);
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

// 関数の引数を取得
// もし関数実装で引数名が与えられない場合はエラー
fn get_func_args(
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Option<(Vec<String>, Vec<Type>, Vec<TokenInfo>)> {
    // 関数宣言か, 関数実装か判断する
    let mut is_func_declaration = false;
    let mut cur = 1;
    while let Some(token) = tokens.get_next(cur) {
        if token.expect_symbol(&Symbol::RightParenthesis) {
            if let Some(token) = tokens.get_next(cur + 1) {
                if token.expect_symbol(&Symbol::LeftCurlyBracket) {
                    is_func_declaration = false;
                }
            }
            break;
        } else {
            cur += 1;
        }
    }

    let mut args_type: Vec<Type> = vec![];
    let mut args_name: Vec<String> = vec![];
    let mut args_info: Vec<TokenInfo> = vec![];

    // consume "("
    tokens.consume_symbol(Symbol::LeftParenthesis);
    while !tokens.expect_symbol(Symbol::RightParenthesis) {
        let (arg_type, arg_name, arg_tokeninfo) = cousume_type_token(tokens, definitions);

        // 変数名なしかつ関数宣言でない
        if arg_name.is_empty() && !is_func_declaration {
            output_unexpected_token_err(tokens);
        }

        args_type.push(arg_type);
        args_name.push(arg_name);
        args_info.push(arg_tokeninfo);
        if tokens.expect_symbol(Symbol::RightParenthesis) {
            break;
        }
        tokens.consume_symbol(Symbol::Comma);
    }

    // consume ")"
    tokens.consume_symbol(Symbol::RightParenthesis);
    if args_info.is_empty() {
        None
    } else {
        Some((args_name, args_type, args_info))
    }
}

fn ast_funcution_implementaion(
    func_name: String,
    func_info: TokenInfo,
    func_type: Type,
    argnames: Option<Vec<String>>,
    args_info: Option<Vec<TokenInfo>>,
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Ast {
    // 関数実装ASTを作成
    definitions.initialize_local_scope();
    let mut args_expr: Option<Vec<Ast>> = None;
    // 引数がある場合
    if let Some(ref argtypes) = func_type.function.as_ref().unwrap().args {
        let mut expr_vec: Vec<Ast> = vec![];
        for (arg_type, (argname, argtoken)) in argtypes.iter().zip(
            argnames
                .unwrap()
                .into_iter()
                .zip(args_info.unwrap().into_iter()),
        ) {
            if let Ok(val) = definitions.declare_local_val(&argname, arg_type.clone()) {
                let type_ = val.get_type();
                let ast = Ast::new_variable_ast(val, argtoken, type_);
                expr_vec.push(ast);
            } else {
                output_alreadydeclared_variable_err(&argtoken);
            }
        }
        args_expr = Some(expr_vec);
    }
    let expfunc_context_ast = ast_exprs(tokens, definitions);
    let frame_size = definitions.get_local_val_frame_size();
    definitions.clear_local_val_scope();
    // 関数AST作成
    Ast::new_function_implementation_ast(
        &func_name,
        func_info,
        func_type,
        frame_size,
        args_expr,
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
    let args = get_func_args(tokens, definitions);
    let mut arg_names = None;
    let mut arg_types = None;
    let mut arg_info = None;
    if let Some(arg_vecs) = args {
        arg_names = Some(arg_vecs.0);
        arg_types = Some(arg_vecs.1);
        arg_info = Some(arg_vecs.2);
    }

    let func_ret;
    if ret_type == definitions.get_type("void").unwrap() {
        func_ret = None;
    } else {
        func_ret = Some(ret_type);
    }

    let func = Function::new(arg_types, func_ret);

    if let Ok(func_type) = definitions.declare_function(&func_name, func) {
        if tokens.expect_symbol(Symbol::SemiColon) {
            tokens.consume_symbol(Symbol::SemiColon);
            None
        } else if tokens.expect_symbol(Symbol::LeftCurlyBracket) {
            if !definitions.can_implement_function(&func_name) {
                output_alreadyimplementedfunction_err(&func_info);
            }
            Some(ast_funcution_implementaion(
                func_name,
                func_info,
                func_type,
                arg_names,
                arg_info,
                tokens,
                definitions,
            ))
        } else {
            output_notsamefunction_err(&func_info);
        }
    } else {
        output_unexpected_token_err(tokens);
    }
}

// グローバル変数定義, 関数宣言, 関数実装を行う
fn ast_global(tokens: &mut Tokens, definitions: &mut Definitions) -> Option<Ast> {
    let (type_, name, info) = cousume_type_token(tokens, definitions);
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
