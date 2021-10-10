use std::process::exit;

use crate::token::parser::*;
use crate::token::token::*;
use crate::SOURCE_TXT;

pub fn exit_no_token_err() -> ! {
    eprintln!("error: no valid token");
    exit(-1);
}

// トークン化に失敗した行とそのトークンを表示して終了する
pub fn exit_parser_error(err: ParserError, parser: &Parser) -> ! {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[parser.line];
    }
    eprintln!("{}", error_line);
    let mut error_cur = " ".repeat(parser.pos);
    error_cur.push('^');
    eprintln!("{}", error_cur);
    eprintln!(
        "line{}, pos{}, error: {}",
        parser.line + 1,
        parser.pos + 1,
        err
    );
    exit(-1);
}

fn get_token(info: &TokenInfo) -> String {
    let error_line;
    unsafe {
        error_line = &SOURCE_TXT[info.line];
    }
    error_line[info.pos..info.pos + info.width].to_string()
}

fn print_token_error_info(err: TokenError, info: &TokenInfo) {
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

pub fn invalid_number_token_err(info: &TokenInfo) -> ! {
    // get invalid token
    let invalidnum_token = get_token(info);
    print_token_error_info(TokenError::InvalidNumberErr(invalidnum_token), info);
    exit(-1);
}

pub fn unexpected_token_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnexpectTokenError, info);
    exit(-1);
}

fn unclosed_token_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnClosedError, info);
    exit(-1);
}

fn unclosed_tokenend_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnClosedEndError, info);
    exit(-1);
}

fn undeclared_variable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnDeclaredVariableError, info);
    exit(-1);
}

fn alreadydeclared_variable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::AlreadyDeclaredVariableError, info);
    exit(-1);
}

pub fn output_unclosed_token_err(tokens: &Tokens) -> ! {
    let err_token;
    if tokens.is_empty() {
        err_token = tokens.get_tail().unwrap();
        unclosed_tokenend_err(&err_token.info);
    } else {
        err_token = tokens.get().unwrap();
        unclosed_token_err(&err_token.info);
    }
}

pub fn output_unexpected_token_err(tokens: &Tokens) -> ! {
    // tokensが何もないときはtokens作成時にerrorとするのでunwrap可能
    let err_token = tokens.get().unwrap();
    unexpected_token_err(&err_token.info);
}

pub fn output_undeclared_variable_err(info: &TokenInfo) -> ! {
    undeclared_variable_err(info);
}

pub fn output_alreadydeclared_variable_err(info: &TokenInfo) -> ! {
    alreadydeclared_variable_err(info);
}