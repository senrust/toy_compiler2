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
    UnexpecdASTKindError(ASTKind, &'static str),
}

impl fmt::Display for ASTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ASTError::UnexpecdASTKindError(ast_type, expected_type) => {
                write!(
                    f,
                    "unexpected ast kind: {:?}, expexcted: {}",
                    ast_type, *expected_type
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Sub,
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
    pub child: Option<Box<AST>>,
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
            child: None,
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
            child: None,
            other: None,
        }
    }
}

fn ast_number(nodes: &mut Nodes, definitions: &mut Definitions) -> Result<AST, ()> {
    if nodes.expect_number() {
        if let Ok((num, info)) = nodes.consume_integer() {
            let type_ = definitions.get_primitive_type(&num);
            match num {
                Number::U64(num_u64) => {
                    return Ok(AST::new_integer_ast(Number::U64(num_u64), info, type_));
                }
                Number::F64(_num_f64) => Err(()),
            }
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

// add = num | (+  num| - num)*
fn ast_add(nodes: &mut Nodes, definitions: &mut Definitions) -> Result<AST, ()> {
    if let Ok(left_ast) = ast_number(nodes, definitions) {
        let mut operation;
        let mut add_ast = left_ast;
        loop {
            if nodes.expect_symbol(Symbol::Add) {
                operation = Operation::Add;
            } else if nodes.expect_symbol(Symbol::Sub) {
                operation = Operation::Sub;
            } else if nodes.is_empty() {
                return Ok(add_ast);
            } else {
                unexpected_node_err(&nodes.get().unwrap().info);
            }

            let add_ast_info = nodes.consume().unwrap();
            if let Ok(right_ast) = ast_number(nodes, definitions) {
                let type_: Rc<Type> = evaluate_binary_operation_type(&add_ast, &right_ast).unwrap();
                add_ast = AST::new_binary_operation_ast(
                    operation,
                    add_ast_info,
                    type_,
                    add_ast,
                    right_ast,
                );
            } else {
                if nodes.is_empty() {
                    unexpected_end_err(&nodes.get_last().unwrap().info);
                } else {
                    unexpected_node_err(&nodes.get().unwrap().info);
                }
            }
        }
    } else {
        unexpected_node_err(&nodes.get().unwrap().info);
    }
}

pub fn make_asts(mut nodes: Nodes) -> Vec<AST> {
    let mut asts: Vec<AST> = vec![];
    let mut programinfo = Definitions::new();
    while nodes.has_node() {
        let ast = ast_add(&mut nodes, &mut programinfo).unwrap();
        asts.push(ast);
    }
    asts
}
