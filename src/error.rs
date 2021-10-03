use std::process::exit;

use crate::interpret_token::{NodeError, NodeInfo};
use crate::tokenizer::{TokenizeError, TokenizeInfo};

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

fn print_error_info(err: NodeError, info: &NodeInfo) {
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
    print_error_info(NodeError::UnexpectNodeError, info);
    exit(-1);
}

pub fn unexpected_end_err(last_info: &NodeInfo) -> ! {
    let last_info = NodeInfo::new(last_info.line, last_info.pos + last_info.width, 0);
    print_error_info(NodeError::UnexpectEndError, &last_info);
    exit(-1);
}
