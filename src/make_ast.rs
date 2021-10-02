use std::collections::HashMap;
use std::rc::Rc;
use crate::definition::types::Type;
use crate::definition::variables::{Variables, Variable};
use crate::interpret_token::{Node, SymbolKind};

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

    fn proceed(&mut self) {
        self.cur += 1;
    }

    fn has_node(&self) -> bool {
        self.cur < self.vec.len()
    }

    fn consume_symbol(&mut self, symbol_kind: SymbolKind) -> bool {
        if self.vec[self.cur].expect_symbol(symbol_kind) {
            self.proceed();
            true    
        } else {
            false
        }
    }

    fn expect_number(&self) -> bool {
        self.vec[self.cur].expect_number()
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

enum ASTKind {
    OperationKind,
    Funcion,
    Variable(Variable),
    ImmidiateInterger(Rc<Type>, u64),
    ImmidiateFloat(Rc<Type>, f64),
} 

struct AST {
    kind: ASTKind,
    type_: Rc<Type>,
    left: Option<Box<AST>>,
    right: Option<Box<AST>>,
    child: Option<Box<AST>>,
    other: Option<Vec<Box<AST>>>,
}