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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Eq,    // ==
    NotEq, // !=
    Gt,    // >
    Lt,    // <
    Ge,    // >=
    Le,    // <=
    Not,   // !
}

#[derive(Debug, Clone)]
pub enum ASTKind {
    FuncionDeclaration(String),
    FuncionCall(Function),
    Operation(Operation),
    LocalVal(LocalVariable),
    GlobalVal(GlobalVariable),
    ImmidiateInterger(Number),
    ImmidiateFloat(Number),
}

#[derive(Debug)]
pub struct AST {
    pub kind: ASTKind,
    pub info: NodeInfo,
    pub type_: Rc<Type>,
    pub left: Option<Box<AST>>,
    pub right: Option<Box<AST>>,
    pub operand: Option<Box<AST>>,
    pub other: Option<Vec<Box<AST>>>,
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
            other: None,
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
            other: None,
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
            other: None,
        }
    }
}

// primary = num | "(" add ")"
fn ast_primary(nodes: &mut Nodes, definitions: &mut Definitions) -> AST {
    if nodes.expect_number() {
        if let Ok((num, info)) = nodes.consume_integer() {
            let type_ = definitions.get_primitive_type(&num);
            match num {
                Number::U64(num_u64) => {
                    return AST::new_integer_ast(Number::U64(num_u64), info, type_);
                }
                Number::F64(_num_f64) => unreachable!(),
            }
        } else {
            // expect_numberでノードが存在することはチェック済みなのでunwrapを使用
            invalidnumber_node_err(&nodes.get().unwrap().info);
        }
    } else if nodes.expect_symbol(Symbol::LeftParenthesis) {
        // drop "(" node
        nodes.consume().unwrap();
        let add_ast = ast_equality(nodes, definitions);
        if nodes.expect_symbol(Symbol::RightParenthesis) {
            // drop ")" node
            nodes.consume().unwrap();
            return add_ast;
        } else {
            output_unexpected_node_err(nodes);
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

// equality   = relational ("==" relational | "!=" relational)*
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

pub fn make_asts(mut nodes: Nodes) -> Vec<AST> {
    let mut asts: Vec<AST> = vec![];
    let mut programinfo = Definitions::new();
    while nodes.has_node() {
        let ast = ast_equality(&mut nodes, &mut programinfo);
        asts.push(ast);
    }
    asts
}
