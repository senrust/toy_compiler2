use crate::interpret_token::{ASTNode, ASTNodeKind, ASTNodeInfo};

enum OperationKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Assign,
    
}

enum ASTKind {
    Operation(OperationKind),
    FanctionCall(String),
}

struct AST {
    kind: ASTKind,
    left: Option<Box<AST>>,
    right: Option<Box<AST>>,
    vec: Option<Vec<AST>>,
}

impl AST {
    fn new(kind: ASTKind) -> Self {
        AST { 
            kind, 
            left: None, 
            right: None, 
            vec: None
        }
    }
}

fn make_ast(ast_nodes: Vec<ASTNode>) -> Vec<AST> {
    let asts: Vec<AST> = vec![];
    asts
}