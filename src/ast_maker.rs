use crate::definition::definitions::Definitions;
use crate::definition::functions::Function;
use crate::definition::number::Number;
use crate::definition::symbols::*;
use crate::definition::types::*;
use crate::definition::variables::*;
use crate::error::*;
use crate::token_interpreter::{NodeInfo, Nodes};
use std::fmt;
use std::rc::Rc;

pub enum ASTError {
    UnexpecdASTKindError(ASTKind, String),
    UnSupportedASTKindError(ASTKind),
    UnAssignableASTKindError,
}

impl fmt::Display for ASTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ASTError::UnexpecdASTKindError(ast_type, expected_type) => {
                write!(
                    f,
                    "unexpected ast kind: {:?}, expexcted type: {}",
                    ast_type, expected_type
                )
            }
            ASTError::UnSupportedASTKindError(ast_type) => {
                write!(f, "unsupported ast kind: {:?}", ast_type,)
            }
            ASTError::UnAssignableASTKindError => {
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
pub enum ASTKind {
    FunctionImplementation((String, usize)),
    FuncionCall(Rc<Function>),
    Expressions,
    Operation(Operation),
    Variable(Variable),
    ImmidiateInterger(Number),
}

#[derive(Debug)]
pub struct AST {
    pub kind: ASTKind,
    pub info: NodeInfo,
    pub type_: Rc<Type>,
    pub left: Option<Box<AST>>,
    pub right: Option<Box<AST>>,
    pub operand: Option<Box<AST>>,
    pub exprs: Option<Vec<AST>>,
    pub context: Option<Box<AST>>,
}

impl AST {
    fn new_integer_ast(num: Number, info: NodeInfo, type_: Rc<Type>) -> AST {
        AST {
            kind: ASTKind::ImmidiateInterger(num),
            info,
            type_,
            left: None,
            right: None,
            operand: None,
            exprs: None,
            context: None,
        }
    }

    fn new_variable_ast(val: Variable, info: NodeInfo, type_: Rc<Type>) -> AST {
        AST {
            kind: ASTKind::Variable(val),
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
        operand: AST,
    ) -> AST {
        AST {
            kind: ASTKind::Operation(operation),
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
        left: AST,
        right: AST,
    ) -> AST {
        AST {
            kind: ASTKind::Operation(operation),
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
        context: AST,
    ) -> AST {
        AST {
            kind: ASTKind::FunctionImplementation((func_name.to_string(), frame_size)),
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
        exprs: Vec<AST>,
        context: Option<Box<AST>>,
    ) -> AST {
        AST {
            kind: ASTKind::Expressions,
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

fn ast_number(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    if !nodes.expect_number() {
        output_unexpected_node_err(nodes);
    }

    if let Ok((num, info)) = nodes.consume_integer() {
        let type_ = definitions.get_primitive_type(&num);
        match num {
            Number::U64(num_u64) => {
                return AST::new_integer_ast(Number::U64(num_u64), info, type_);
            }
            Number::F64(_num_f64) => unreachable!(),
        }
    } else {
        invalid_number_node_err(&nodes.get().unwrap().info);
    }
}

fn ast_variable(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
        let val_ast = AST::new_variable_ast(val, info, val_type);
        return val_ast;
    } else {
        output_unexpected_node_err(nodes);
    }
}

// primary = num | variable | "(" add ")"
fn ast_primary(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
            return add_ast;
        } else {
            output_unclosed_node_err(nodes);
        }
    } else {
        output_unexpected_node_err(nodes);
    }
}

// unary = primary |  + primary |  - primary | ! unary
fn ast_unary(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    if nodes.expect_symbol(Symbol::Add) {
        // drop "+" node
        nodes.consume().unwrap();
        return ast_primary(nodes, definitions);
    } else if nodes.expect_symbol(Symbol::Sub) {
        // drop "-" node
        let sub_info = nodes.consume().unwrap();
        let primary_ast = ast_primary(nodes, definitions);
        let type_ = primary_ast.type_.clone();
        let zero_ast = AST::new_integer_ast(Number::U64(0), sub_info.clone(), type_.clone());
        let sub_ast =
            AST::new_binary_operation_ast(Operation::Sub, sub_info, type_, zero_ast, primary_ast);
        return sub_ast;
    } else if nodes.expect_symbol(Symbol::Not) {
        // drop "!" node
        let not_info = nodes.consume().unwrap();
        let operand_ast = ast_unary(nodes, definitions);
        // とりあえず8バイトにしておく
        let type_ = definitions.get_type("long").unwrap();
        let not_ast = AST::new_single_operation_ast(Operation::Not, not_info, type_, operand_ast);
        return not_ast;
    } else {
        return ast_primary(nodes, definitions);
    }
}

// mul = unary | (* unary | / unary)*
fn ast_mul(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
        mul_ast = AST::new_binary_operation_ast(operation, ast_info, type_, mul_ast, right_ast);
    }
}

// add = mul | (+  mul | - mul)*
fn ast_add(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
        add_ast = AST::new_binary_operation_ast(operation, ast_info, type_, add_ast, right_ast);
    }
}

// relational = add (">" add | "<" add | ">=" add| "<=" add)*
fn ast_relational(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
            AST::new_binary_operation_ast(operation, ast_info, type_, relational_ast, right_ast);
    }
}

// equality = relational ("==" relational | "!=" relational)*
fn ast_equality(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
            AST::new_binary_operation_ast(operation, ast_info, type_, equality_ast, right_ast);
    }
}

// assign = equality ("=" equality)*
// 左辺値が左辺値となりうるかの確認はコンパイル側でおこなう
fn ast_assign(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
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
        assign_ast = AST::new_binary_operation_ast(
            Operation::Assign,
            ast_info,
            type_,
            assign_ast,
            ast_assigner,
        );
    }
}

// expr = assign | exprs
fn ast_expr(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    if nodes.expect_symbol(Symbol::LeftCurlyBracket) {
        ast_exprs(nodes, definitions)
    } else {
        ast_assign(nodes, definitions)
    }
}

// exprs = "{" (expr ";") *  + (expr)? "}"
fn ast_exprs(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    let mut exprs: Vec<AST> = vec![];

    if !nodes.expect_symbol(Symbol::LeftCurlyBracket) {
        output_unexpected_node_err(nodes);
    }

    // consume "{"
    nodes.consume().unwrap();
    // ローカル変数のネストを深くする
    definitions.enter_new_local_scope();

    let mut exd_context: Option<Box<AST>> = None; // {expr; expr; expr} の";"で閉じられていない最後のexpr
                                                  // "}"が登場するまで
    while !nodes.expect_symbol(Symbol::RightCurlyBracket) {
        if nodes.is_empty() {
            output_unclosed_node_err(nodes);
        }
        let expr = ast_expr(nodes, definitions);
        if nodes.expect_symbol(Symbol::RightCurlyBracket) {
            exd_context = Some(Box::new(expr));
        } else if nodes.expect_symbol(Symbol::SemiColon) {
            // consume ";"
            nodes.consume().unwrap();
            exprs.push(expr);
        } else if expr.kind == ASTKind::Expressions {
            // 複文のときはセミコロン不要
            exprs.push(expr);
        } else {
            output_unexpected_node_err(nodes);
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

    let exprs_ast = AST::new_expressions_ast(exprs_info, exprs_type, exprs, exd_context);
    exprs_ast
}

fn ast_funcution_implementaion(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    // テンポラリとしてmain関数を定義しておく
    // 今後関数情報作成部を実装する
    let main_func = Function::new("main", None, None);
    let func_type = definitions.declear_function("main", main_func).unwrap();
    let info = NodeInfo::new(0, 0, 0);

    definitions.initialize_local_scope();
    let expfunc_context_ast = ast_exprs(nodes, definitions);
    let frame_size = definitions.get_local_val_frame_size();
    let func_ast = AST::new_function_implementation_ast(
        "main",
        info,
        func_type,
        frame_size,
        expfunc_context_ast,
    );
    func_ast
}

pub fn make_asts(mut nodes: Nodes) -> Vec<AST> {
    let mut asts: Vec<AST> = vec![];
    let mut programinfo = Definitions::new();
    while nodes.has_node() {
        let ast = ast_funcution_implementaion(&mut nodes, &mut programinfo);
        asts.push(ast);
    }
    asts
}
