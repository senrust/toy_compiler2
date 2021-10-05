use std::process::exit;

use crate::ast_maker::{ASTError, AST};
use crate::source_tokenizer::{TokenizeError, TokenizeInfo};
use crate::token_interpreter::{NodeError, NodeInfo};

use crate::SOURCE_TXT;

// トークン化に失敗した行とそのトークンを表示して終了する
pub fn tokenizer_error(err: TokenizeError, info: &TokenizeInfo) -> ! {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[info.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(info.pos);
    error_cur.push_str("^");
    eprintln!("{}", error_cur);
    eprintln!("line{}, pos{}, error: {}", info.line + 1, info.pos + 1, err);
    exit(-1);
}

fn print_node_error_info(err: NodeError, info: &NodeInfo) {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[info.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(info.pos);
    error_cur.push_str("^");
    eprintln!("{}", error_cur);
    eprintln!("line{}, pos{}, error: {}", info.line + 1, info.pos + 1, err);
}

pub fn unexpected_node_err(info: &NodeInfo) -> ! {
    print_node_error_info(NodeError::UnexpectNodeError, info);
    exit(-1);
}

pub fn unexpected_end_err(last_info: &NodeInfo) -> ! {
    let last_info = NodeInfo::new(last_info.line, last_info.pos + last_info.width, 0);
    print_node_error_info(NodeError::UnexpectEndError, &last_info);
    exit(-1);
}

fn print_ast_error_info(ast: &AST, err: ASTError) {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[ast.info.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(ast.info.pos);
    error_cur.push_str("^");
    eprintln!("{}", error_cur);
    eprintln!(
        "line{}, pos{}, error: {}",
        ast.info.line + 1,
        ast.info.pos + 1,
        err
    );
    if cfg!(debug_assertions) {
        eprintln!("Err AST: {:?}", ast);
    }
}

pub fn unexpected_ast_err(ast: &AST, expected_kind: String) -> ! {
    print_ast_error_info(
        &ast,
        ASTError::UnexpecdASTKindError(ast.kind.clone(), expected_kind),
    );
    exit(-1);
}
