use std::fmt;
use std::process::exit;

use crate::ast::ast::{Ast, AstKind};
use crate::SOURCE_TXT;

pub enum AstError {
    InValidDirection(String),
    UnExpectedAs(AstKind, String),
    UnSupportedAst(AstKind),
    UnAssignableAst,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstError::InValidDirection(direction) => {
                write!(f, "can not use {} here", direction)
            }
            AstError::UnExpectedAs(ast_type, expected_type) => {
                write!(
                    f,
                    "unexpected ast kind: {:?}, expexcted type: {}",
                    ast_type, expected_type
                )
            }
            AstError::UnSupportedAst(ast_type) => {
                write!(f, "unsupported ast kind: {:?}", ast_type,)
            }
            AstError::UnAssignableAst => {
                write!(f, "this tokein cant not be assigned")
            }
        }
    }
}

fn print_ast_error_info(ast: &Ast, err: AstError) {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[ast.info.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(ast.info.pos);
    error_cur.push('^');
    eprintln!("{}", error_cur);
    eprintln!(
        "line{}, pos{}, error: {}",
        ast.info.line + 1,
        ast.info.pos + 1,
        err
    );
    if cfg!(debug_assertions) {
        eprintln!("Err Ast: {:?}", ast);
    }
}

pub fn invalid_direction_err(ast: &Ast, direction: &str) -> ! {
    print_ast_error_info(ast, AstError::InValidDirection(direction.to_string()));
    exit(-1);
}

pub fn unexpected_ast_err(ast: &Ast, expected_kind: &str) -> ! {
    print_ast_error_info(
        ast,
        AstError::UnExpectedAs(ast.kind.clone(), expected_kind.to_string()),
    );
    exit(-1);
}

pub fn unsupported_ast_err(ast: &Ast) -> ! {
    print_ast_error_info(ast, AstError::UnSupportedAst(ast.kind.clone()));
    exit(-1);
}

pub fn unassignable_ast_err(ast: &Ast) -> ! {
    print_ast_error_info(ast, AstError::UnAssignableAst);
    exit(-1);
}
