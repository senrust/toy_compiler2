use std::rc::Rc;
use crate::definition::types::*;
use crate::definition::variables::{Variable, Variables};
use crate::definition::functions::{Function, Functions};
use crate::definition::number::Number;
use crate::error::*;
use crate::interpret_token::{Node, NodeError, SymbolKind};

struct Nodes {
    vec: Vec<Node>,
    cur: usize,
}

impl Nodes {
    fn new(node_vec: Vec<Node>) -> Self {
        Nodes {
            vec: node_vec,
            cur: 0,
        }
    }

    fn get(&self) -> Option<&Node> {
        self.vec.get(self.cur)
    }

    fn get_last(&self) -> Option<&Node> {
        self.vec.last()
    }

    fn proceed(&mut self) {
        self.cur += 1;
    }

    fn is_empty(&self) -> bool {
        self.cur >= self.vec.len()
    }

    fn has_node(&self) -> bool {
        self.cur < self.vec.len()
    }

    fn expect_symbols(&mut self, symbol_kinds: &[SymbolKind]) -> bool {
        for symbol_kind in symbol_kinds {
            if let Some(node) = self.vec.get(self.cur) {
                if node.expect_symbol(symbol_kind) {
                    return true;
                } 
            } 
        }
        false
    }

    fn consume_symbol(&mut self, symbol_kind: SymbolKind) -> bool {
        if let Some(node) = self.vec.get(self.cur) {
            if node.expect_symbol(&symbol_kind) {
                self.cur += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn expect_number(&self) -> bool {
        if let Some(node) = self.vec.get(self.cur) {
            node.expect_number() 
        } else {
            false
        }
    }

    fn consume_integer(&mut self) -> Result<Number, ()> {
        if let Some(node) = self.vec.get(self.cur) {
            if let Ok(num) = node.get_interger() {
                self.cur += 1;
                Ok(num)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

enum OperationKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Assign,  
}

pub enum ASTKind {
    FuncionFefinition,
    Operation(OperationKind),
    FuncionCall,
    Variable(Variable),
    ImmidiateInterger(Rc<Type>, u64),
    ImmidiateFloat(Rc<Type>, f64),
}

pub struct AST {
    pub kind: ASTKind,
    pub type_: Rc<Type>,
    pub left: Option<Box<AST>>,
    pub right: Option<Box<AST>>,
    pub child: Option<Box<AST>>,
    pub other: Option<Vec<Box<AST>>>,
}

impl AST {
    fn new_integer_ast(num: u64, type_: Rc<Type>) -> AST {
        AST { kind: ASTKind::ImmidiateInterger(type_.clone(), num), type_, left: None, right: None, child: None, other: None }
    }

    fn new_binary_operation_ast(operation: OperationKind, type_: Rc<Type>, left: AST, right: AST) -> AST {
        AST { kind: ASTKind::Operation(operation), type_, left: Some(Box::new(left)), right: Some(Box::new(right)), child: None, other: None }
    }
}

fn ast_number(nodes: &mut Nodes, types: &mut Types, variables: &mut Variables, functions: &mut Functions) -> Result<AST, ()> {
    if nodes.expect_number() {
        if let  Ok(num) = nodes.consume_integer() {
            let type_ =types.get_iimidiate_type(&num);
            match num {
                Number::U64(num_u64) => {
                    return Ok(AST::new_integer_ast(num_u64, type_));
                }
                Number::F64(num_f64) => {
                    Err(())
                }
            }
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

// add = num | (+  num| - num)*
fn ast_add(nodes: &mut Nodes, types: &mut Types, variables: &mut Variables, functions: &mut Functions) -> Result<AST, ()> {
    if let Ok(left_number_ast) = ast_number(nodes, types, variables, functions) {
        let mut operation_kind;
        let mut add_ast = left_number_ast;
        loop {
            if nodes.consume_symbol(SymbolKind::Add) { 
                operation_kind = OperationKind::Add; 
            } else if nodes.consume_symbol(SymbolKind::Sub) {
                operation_kind = OperationKind::Sub; 
            } else if nodes.is_empty() {
                return  Ok(add_ast);
            } else {
                unexpected_node_err(&nodes.get().unwrap().info);
            }
            if let Ok(right_number_ast) = ast_number(nodes, types, variables, functions) {
                let type_: Rc<Type> = evaluate_binary_operation_type(&add_ast, &right_number_ast);
                add_ast = AST::new_binary_operation_ast(operation_kind,  type_, add_ast, right_number_ast);
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

pub fn make_asts(node_vec: Vec<Node>) -> Vec<AST>{
    let mut nodes = Nodes::new(node_vec);
    let mut asts: Vec<AST> = vec![];
    let mut types = Types::new();
    let mut variables = Variables::new();
    let mut functions = Functions::new();
    while nodes.has_node() {
        let ast = ast_add(&mut nodes, &mut types, &mut variables, &mut functions).unwrap();
        asts.push(ast);
    }
    asts
}