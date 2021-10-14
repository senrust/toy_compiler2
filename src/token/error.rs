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

fn unclosed_token_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnClosed, info);
    exit(-1);
}

fn unclosed_tokenend_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::ReachEndWithoutClose, info);
    exit(-1);
}

pub fn invalid_number_token_err(info: &TokenInfo) -> ! {
    // get invalid token
    let invalidnum_token = get_token(info);
    print_token_error_info(TokenError::InvalidNumber(invalidnum_token), info);
    exit(-1);
}

pub fn unexpected_token_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnexpectToken, info);
    exit(-1);
}

pub fn notenough_token_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::NotEnoughToken, info);
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
    if let Some(err_token) = tokens.get() {
        unexpected_token_err(&err_token.info);
    } else {
        let prev_token = tokens.get_prev(1).unwrap();
        notenough_token_err(&prev_token.info);
    }
}

pub fn output_undeclared_variable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UndeclaredVariable, info);
    exit(-1);
}

pub fn output_alreadydeclared_variable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::AlreadyDeclaredVariable, info);
    exit(-1);
}

pub fn output_undefinedfunction_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UndefinedFunctionCall, info);
    exit(-1);
}

pub fn output_notsamefunction_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::NotSameFunction, info);
    exit(-1);
}

pub fn output_alreadyimplementedfunction_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::AlreadyImplementedFunction, info);
    exit(-1);
}

pub fn output_incorrectarg_err(tokens: &Tokens) -> ! {
    if let Some(err_token) = tokens.get() {
        print_token_error_info(TokenError::InCorrectArgs, &err_token.info);
        exit(-1);
    } else {
        let prev_token = tokens.get_prev(1).unwrap();
        notenough_token_err(&prev_token.info);
    }
}

pub fn output_defferenttype_err(tokens: &Tokens) -> ! {
    if let Some(err_token) = tokens.get() {
        print_token_error_info(TokenError::DefferentType, &err_token.info);
        exit(-1);
    } else {
        let prev_token = tokens.get_prev(1).unwrap();
        notenough_token_err(&prev_token.info);
    }
}

pub fn output_undereferensable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnDereferensable, info);
    exit(-1);
}

pub fn output_unaddressable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::Unaddressable, info);
    exit(-1);
}

pub fn output_notinteger_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::NotInteger, info);
    exit(-1);
}

pub fn output_unindexiable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnIndexiable, info);
    exit(-1);
}

pub fn output_unexecutable_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::UnExecutable, info);
    exit(-1);
}

pub fn output_different_returntype_err(info: &TokenInfo) -> ! {
    print_token_error_info(TokenError::DifferentReturnType, info);
    exit(-1);
}
