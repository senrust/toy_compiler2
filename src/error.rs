use std::process::exit;

use crate::ast_maker::{Ast, AstError};
use crate::source_tokenizer::{TokenizeError, TokenizeInfo};
use crate::token_interpreter::{NodeError, NodeInfo, Nodes};

use crate::SOURCE_TXT;

fn get_token(info: &NodeInfo) -> String {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[info.line];
    }
    error_line[info.pos..info.pos + info.width].to_string()
}

// トークン化に失敗した行とそのトークンを表示して終了する
pub fn exit_tokenizer_error(err: TokenizeError, info: &TokenizeInfo) -> ! {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[info.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(info.pos);
    error_cur.push('^');
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
    error_cur.push('^');
    eprintln!("{}", error_cur);
    eprintln!("line{}, pos{}, error: {}", info.line + 1, info.pos + 1, err);
}

pub fn invalid_number_node_err(info: &NodeInfo) -> ! {
    // get invalid token
    let invalidnum_token = get_token(info);
    print_node_error_info(NodeError::InvalidNumberErr(invalidnum_token), info);
    exit(-1);
}

fn unexpected_node_err(info: &NodeInfo) -> ! {
    print_node_error_info(NodeError::UnexpectNodeError, info);
    exit(-1);
}

pub fn exit_no_token_err() -> ! {
    eprintln!("error: no valid token");
    exit(-1);
}

fn unclosed_node_err(info: &NodeInfo) -> ! {
    print_node_error_info(NodeError::UnClosedError, info);
    exit(-1);
}

fn unclosed_nodeend_err(info: &NodeInfo) -> ! {
    print_node_error_info(NodeError::UnClosedEndError, info);
    exit(-1);
}

pub fn output_unclosed_node_err(nodes: &Nodes) -> ! {
    let err_node;
    if nodes.is_empty() {
        err_node = nodes.get_tail().unwrap();
        unclosed_nodeend_err(&err_node.info);
    } else {
        err_node = nodes.get().unwrap();
        unclosed_node_err(&err_node.info);
    }
}

pub fn output_unexpected_node_err(nodes: &Nodes) -> ! {
    // nodesが何もないときはnodes作成時にerrorとするのでunwrap可能
    let err_node = nodes.get().unwrap();
    unexpected_node_err(&err_node.info);
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

pub fn unexpected_ast_err(ast: &Ast, expected_kind: String) -> ! {
    print_ast_error_info(
        ast,
        AstError::UnExpectedAstKind(ast.kind.clone(), expected_kind),
    );
    exit(-1);
}

pub fn unsupported_ast_err(ast: &Ast) -> ! {
    print_ast_error_info(ast, AstError::UnSupportedAstKind(ast.kind.clone()));
    exit(-1);
}

pub fn unassignable_ast_err(ast: &Ast) -> ! {
    print_ast_error_info(ast, AstError::UnAssignableAstKind);
    exit(-1);
}
