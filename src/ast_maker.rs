use crate::definition::definitions::Definitions;
use crate::definition::functions::Function;
use crate::definition::number::Number;
use crate::definition::reservedwords::*;
use crate::definition::symbols::*;
use crate::definition::types::*;
use crate::definition::variables::*;
use crate::error::*;
use crate::token_interpreter::{NodeInfo, Nodes};
use std::fmt;
use std::rc::Rc;

pub enum AstError {
    UnExpectedAstKind(AstKind, String),
    UnSupportedAstKind(AstKind),
    UnAssignableAstKind,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstError::UnExpectedAstKind(ast_type, expected_type) => {
                write!(
                    f,
                    "unexpected ast kind: {:?}, expexcted type: {}",
                    ast_type, expected_type
                )
            }
            AstError::UnSupportedAstKind(ast_type) => {
                write!(f, "unsupported ast kind: {:?}", ast_type,)
            }
            AstError::UnAssignableAstKind => {
                write!(f, "this tokein cant not be assigned")
            }
        }
    }
}

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
    FuncionCall(Rc<Function>),
    Expressions,
    Operation(Operation),
    Control(Control),
    Variable(Variable),
    ImmidiateInterger(Number),
}

#[derive(Debug)]
pub struct Ast {
    pub kind: AstKind,
    pub info: NodeInfo,
    pub type_: Rc<Type>,
    pub left: Option<Box<Ast>>,
    pub right: Option<Box<Ast>>,
    pub operand: Option<Box<Ast>>,
    pub exprs: Option<Vec<Ast>>,
    pub context: Option<Box<Ast>>,
}

impl Ast {
    fn new_integer_ast(num: Number, info: NodeInfo, type_: Rc<Type>) -> Ast {
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

    fn new_variable_ast(val: Variable, info: NodeInfo, type_: Rc<Type>) -> Ast {
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
        info: NodeInfo,
        type_: Rc<Type>,
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
        info: NodeInfo,
        type_: Rc<Type>,
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
        info: NodeInfo,
        type_: Rc<Type>,
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
        info: NodeInfo,
        type_: Rc<Type>,
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
        info: NodeInfo,
        type_: Rc<Type>,
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

fn ast_number(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if !nodes.expect_number() {
        output_unexpected_node_err(nodes);
    }

    if let Ok((num, info)) = nodes.consume_integer() {
        let type_ = definitions.get_primitive_type(&num);
        match num {
            Number::U64(num_u64) => Ast::new_integer_ast(Number::U64(num_u64), info, type_),
            Number::F64(_num_f64) => unreachable!(),
        }
    } else {
        invalid_number_node_err(&nodes.get().unwrap().info);
    }
}

fn ast_variable(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if !nodes.expect_identifier() {
        output_unexpected_node_err(nodes);
    }

    if let Ok((ident, info)) = nodes.consume_identifier() {
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
        output_unexpected_node_err(nodes);
    }
}

// primary = num | variable | "(" add ")"
fn ast_primary(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if nodes.expect_number() {
        ast_number(nodes, definitions)
    } else if nodes.expect_identifier() {
        ast_variable(nodes, definitions)
    } else if nodes.expect_symbol(Symbol::LeftParenthesis) {
        // drop "(" node
        nodes.consume().unwrap();
        let add_ast = ast_equality(nodes, definitions);
        if nodes.expect_symbol(Symbol::RightParenthesis) {
            // drop ")" node
            nodes.consume().unwrap();
            add_ast
        } else {
            output_unclosed_node_err(nodes);
        }
    } else {
        output_unexpected_node_err(nodes);
    }
}

// unary = primary |  + primary |  - primary | ! unary
fn ast_unary(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if nodes.expect_symbol(Symbol::Add) {
        // drop "+" node
        nodes.consume().unwrap();
        ast_primary(nodes, definitions)
    } else if nodes.expect_symbol(Symbol::Sub) {
        // drop "-" node
        let sub_info = nodes.consume().unwrap();
        let primary_ast = ast_primary(nodes, definitions);
        let type_ = primary_ast.type_.clone();
        let zero_ast = Ast::new_integer_ast(Number::U64(0), sub_info.clone(), type_.clone());
        Ast::new_binary_operation_ast(Operation::Sub, sub_info, type_, zero_ast, primary_ast)
    } else if nodes.expect_symbol(Symbol::Not) {
        // drop "!" node
        let not_info = nodes.consume().unwrap();
        let operand_ast = ast_unary(nodes, definitions);
        // とりあえず8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        Ast::new_single_operation_ast(Operation::Not, not_info, type_, operand_ast)
    } else {
        ast_primary(nodes, definitions)
    }
}

// mul = unary | (* unary | / unary)*
fn ast_mul(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_unary(nodes, definitions);
    let mut operation;
    let mut mul_ast = left_ast;
    loop {
        if nodes.expect_symbol(Symbol::Mul) {
            operation = Operation::Mul;
        } else if nodes.expect_symbol(Symbol::Div) {
            operation = Operation::Div;
        } else {
            return mul_ast;
        }

        let ast_info = nodes.consume().unwrap();
        let right_ast = ast_unary(nodes, definitions);
        let type_: Rc<Type> = evaluate_binary_operation_type(&mul_ast, &right_ast).unwrap();
        mul_ast = Ast::new_binary_operation_ast(operation, ast_info, type_, mul_ast, right_ast);
    }
}

// add = mul | (+  mul | - mul)*
fn ast_add(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_mul(nodes, definitions);
    let mut operation;
    let mut add_ast = left_ast;
    loop {
        if nodes.expect_symbol(Symbol::Add) {
            operation = Operation::Add;
        } else if nodes.expect_symbol(Symbol::Sub) {
            operation = Operation::Sub;
        } else {
            return add_ast;
        }

        let ast_info = nodes.consume().unwrap();
        let right_ast = ast_mul(nodes, definitions);
        let type_: Rc<Type> = evaluate_binary_operation_type(&add_ast, &right_ast).unwrap();
        add_ast = Ast::new_binary_operation_ast(operation, ast_info, type_, add_ast, right_ast);
    }
}

// relational = add (">" add | "<" add | ">=" add| "<=" add)*
fn ast_relational(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_add(nodes, definitions);
    let mut operation;
    let mut relational_ast = left_ast;
    loop {
        if nodes.expect_symbol(Symbol::Gt) {
            operation = Operation::Gt;
        } else if nodes.expect_symbol(Symbol::Lt) {
            operation = Operation::Lt;
        } else if nodes.expect_symbol(Symbol::Ge) {
            operation = Operation::Ge;
        } else if nodes.expect_symbol(Symbol::Le) {
            operation = Operation::Le;
        } else {
            return relational_ast;
        }

        let ast_info = nodes.consume().unwrap();
        let right_ast = ast_add(nodes, definitions);
        // とりあえず比較の型は8バイトにしておく
        let type_: Rc<Type> = definitions.get_type("long").unwrap();
        relational_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, relational_ast, right_ast);
    }
}

// equality = relational ("==" relational | "!=" relational)*
fn ast_equality(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let left_ast = ast_relational(nodes, definitions);
    let mut operation;
    let mut equality_ast = left_ast;
    loop {
        if nodes.expect_symbol(Symbol::Eq) {
            operation = Operation::Eq;
        } else if nodes.expect_symbol(Symbol::NotEq) {
            operation = Operation::NotEq;
        } else {
            return equality_ast;
        }

        let ast_info = nodes.consume().unwrap();
        let right_ast = ast_add(nodes, definitions);
        // とりあえず比較の型は8バイトにしておく
        let type_: Rc<Type> = definitions.get_type("long").unwrap();
        equality_ast =
            Ast::new_binary_operation_ast(operation, ast_info, type_, equality_ast, right_ast);
    }
}

// assign = equality ("=" equality)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
fn ast_assign(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let assignee_ast = ast_equality(nodes, definitions);
    let mut assign_ast = assignee_ast;
    loop {
        if !nodes.expect_symbol(Symbol::Assign) {
            return assign_ast;
        }
        let ast_info = nodes.consume().unwrap();
        let ast_assigner = ast_assign(nodes, definitions);
        // とりあえず代入の型は8バイトにしておく
        let type_: Rc<Type> = definitions.get_type("long").unwrap();
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
fn ast_return(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if !nodes.expect_reserved(Reserved::Return) {
        output_unexpected_node_err(nodes);
    }
    // consume "return"
    let info = nodes.consume().unwrap();
    let return_value = ast_assign(nodes, definitions);
    let type_ = return_value.type_.clone();
    // 今後関数の定義されている戻り型と比較を行う
    // 即;ならばvoid型に設定する
    let context = vec![return_value];
    Ast::new_control_ast(info, type_, Control::Return, None, context)
}

// if = "if" "(" assign ")" expr ("else" expr)?
// if は contextに条件式, exprs[0]に trueのAst, exprs[1]にfalseのAstが入る
fn ast_if(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if !nodes.expect_reserved(Reserved::If) {
        output_unexpected_node_err(nodes);
    }

    let mut if_ast_vec: Vec<Ast> = vec![];
    // consume "if"
    let if_info = nodes.consume().unwrap();
    let if_type = definitions.get_type("void").unwrap();
    if !nodes.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_node_err(nodes);
    }
    // consume "("
    nodes.consume().unwrap();
    let condition_ast = ast_assign(nodes, definitions);
    if !nodes.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_node_err(nodes);
    }
    // consume ")"
    nodes.consume().unwrap();
    // true時のAst
    let true_ast = ast_expr(nodes, definitions);
    if_ast_vec.push(true_ast);
    if nodes.expect_reserved(Reserved::Else) {
        // consume "else"
        nodes.consume().unwrap();
        let else_ast = ast_expr(nodes, definitions);
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
fn ast_expr(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if nodes.expect_symbol(Symbol::LeftCurlyBracket) {
        ast_exprs(nodes, definitions)
    } else if nodes.expect_reserved(Reserved::Return) {
        ast_return(nodes, definitions)
    } else if nodes.expect_reserved(Reserved::If) {
        ast_if(nodes, definitions)
    } else {
        ast_assign(nodes, definitions)
    }
}

// exprs = "{" (expr ";") *  + (expr)? "}"
fn ast_exprs(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    let mut exprs: Vec<Ast> = vec![];

    if !nodes.expect_symbol(Symbol::LeftCurlyBracket) {
        output_unexpected_node_err(nodes);
    }

    // consume "{"
    nodes.consume().unwrap();
    // ローカル変数のネストを深くする
    definitions.enter_new_local_scope();

    let exd_context: Option<Box<Ast>> = None;
    while !nodes.expect_symbol(Symbol::RightCurlyBracket) {
        if nodes.is_empty() {
            output_unclosed_node_err(nodes);
        }
        let expr = ast_expr(nodes, definitions);
        if nodes.expect_symbol(Symbol::SemiColon) {
            // consume ";"
            nodes.consume().unwrap();
            exprs.push(expr);
        } else {
            // 複文, if文はセミコロン不要
            match &expr.kind {
                AstKind::Expressions => exprs.push(expr),
                AstKind::Control(Control::If) => exprs.push(expr),
                _ => output_unexpected_node_err(nodes),
            }
        }
    }
    // ローカル変数のスコープを抜ける
    definitions.exit_current_local_scope();

    // "}" の位置を複文の情報とする
    let exprs_info = nodes.consume().unwrap();
    let exprs_type; // 複文が返す型情報
    if let Some(context) = &exd_context {
        exprs_type = context.type_.clone();
    } else {
        // 何も返さない場合はvoid型にしておく
        exprs_type = definitions.get_type("void").unwrap();
    }

    Ast::new_expressions_ast(exprs_info, exprs_type, exprs, exd_context)
}

// 関数の引数を取得します
fn get_func_args(nodes: &mut Nodes, _definitions: &mut Definitions) -> Option<Vec<Rc<Type>>> {
    if !nodes.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_node_err(nodes);
    }
    // drop "("
    nodes.consume().unwrap();

    if !nodes.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_node_err(nodes);
    }
    // drop ")"
    nodes.consume().unwrap();
    None
}

fn ast_funcution_implementaion(nodes: &mut Nodes, definitions: &mut Definitions) -> Ast {
    if !nodes.expect_identifier() {
        output_unexpected_node_err(nodes);
    }
    // 関数定義
    let (func_name, info) = nodes.consume_identifier().unwrap();
    let func_args = get_func_args(nodes, definitions);
    let func = Function::new(&func_name, func_args, None);
    let func_type = definitions.declear_function(&func_name, func).unwrap();
    // 関数実装ASTを作成
    definitions.initialize_local_scope();
    let expfunc_context_ast = ast_exprs(nodes, definitions);
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

pub fn make_asts(mut nodes: Nodes) -> Vec<Ast> {
    let mut asts: Vec<Ast> = vec![];
    let mut programinfo = Definitions::new();
    while nodes.has_node() {
        let ast = ast_funcution_implementaion(&mut nodes, &mut programinfo);
        asts.push(ast);
    }
    asts
}
