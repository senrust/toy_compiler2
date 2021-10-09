use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::token::error::*;
use crate::SOURCE_TXT;

macro_rules! symbols_without_dot_or_space {
    () => {
        '+' | '-'
            | '/'
            | '*'
            | '='
            | '&'
            | '^'
            | '<'
            | '>'
            | '|'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | ';'
            | ','
            | '!'
            | '~'
            | '#'
    };
}

macro_rules! twochars_symbol_array {
    () => {
        [
            "<<", ">>", "++", "--", "==", "!=", "<=", ">=", "||", "&&", "+=", "-=", "*=", "/=",
            "%=", "&=", "^=", "|=", "->",
        ]
    };
}

macro_rules! threechars_symbol_array {
    () => {
        ["<<=", ">>="]
    };
}

// トークン種別
// ただしNumberは妥当な数字の文字列かチェックしていない
pub enum RawTokenKind {
    Identifier,
    Number,
    Symbol,
    QuoteText,
}

pub struct RawToken {
    pub rawtoken: String,
    pub kind: RawTokenKind,
    pub line: usize,
    pub pos: usize,
}

impl RawToken {
    fn new(rawtoken: &[char], parser: &Parser, kind: RawTokenKind) -> RawToken {
        RawToken {
            rawtoken: rawtoken.iter().collect(),
            kind,
            line: parser.line,
            pos: parser.pos,
        }
    }

    fn new_number_rawtoken(rawtoken: &[char], parser: &Parser) -> RawToken {
        let rawtoken_string: String = rawtoken.iter().collect();
        RawToken {
            rawtoken: rawtoken_string,
            kind: RawTokenKind::Number,
            line: parser.line,
            pos: parser.pos,
        }
    }
}

pub enum ParserError {
    InvalidIdentifiler(String),
    UnClosedError,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::InvalidIdentifiler(identifier) => {
                write!(f, "{} is invalid identifier", identifier)
            }
            ParserError::UnClosedError => {
                write!(f, "not closed")
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ParserState {
    Empty,
    Comment,
    LineComment,
    Identifier,
    QuoteText,
    Number,
    Symbol,
}

pub struct Parser {
    pub state: ParserState,
    pub line: usize,
    pub pos: usize,
}

impl Parser {
    fn new() -> Self {
        Parser {
            state: ParserState::Empty,
            line: 0,
            pos: 0,
        }
    }
}

struct LineParser {
    line: Vec<char>,
    cur: usize,
}

impl LineParser {
    fn new(line: Vec<char>) -> LineParser {
        LineParser { line, cur: 0 }
    }

    fn get_pos(&self) -> usize {
        self.cur
    }

    fn has_char(&self) -> bool {
        self.cur < self.line.len()
    }

    fn peek_nextchar(&self, step: usize) -> Option<char> {
        self.line.get(self.cur + step).cloned()
    }

    fn peek_char(&self) -> char {
        self.line[self.cur]
    }

    fn get_char(&mut self) -> char {
        let ch = self.line[self.cur];
        self.cur += 1;
        ch
    }

    fn proceed(&mut self) {
        self.cur += 1;
    }

    fn advance(&mut self, step: usize) {
        self.cur += step;
    }

    fn compare_text(&self, text: &str) -> bool {
        for (i, j) in text.chars().zip(self.line[self.cur..].iter()) {
            if i != *j {
                return false;
            }
        }
        true
    }
}

// トークン種別判定
fn initialize_parser(
    parse_line: &mut LineParser,
    rawtoken_chars: &mut Vec<char>,
) -> Result<ParserState, ()> {
    // 1行コメントチェック
    if parse_line.compare_text("//") {
        parse_line.advance(2);
        return Ok(ParserState::LineComment);
    }

    // 複数行コメントチェック
    if parse_line.compare_text("/*") {
        parse_line.advance(2);
        return Ok(ParserState::Comment);
    }

    let ch = parse_line.get_char();
    if ch == ' ' {
        return Ok(ParserState::Empty);
    }

    rawtoken_chars.push(ch);
    if ch.is_ascii_digit() {
        return Ok(ParserState::Number);
    }

    match ch {
        symbols_without_dot_or_space!() | '.' => {
            return Ok(ParserState::Symbol);
        }
        _ => {}
    }

    if ch.is_ascii_alphabetic() || ch == '_' {
        return Ok(ParserState::Identifier);
    }

    if ch == '"' || ch == '\'' {
        return Ok(ParserState::QuoteText);
    }

    Err(())
}

// コメントが閉じられるまでループ
// コメントが閉じられない場合は複数行コメントとして次の行で閉じられるまで再ループを行う
fn proceed_until_comment_closed(parse_line: &mut LineParser) -> bool {
    let mut ch = parse_line.get_char();
    while parse_line.has_char() {
        let next_ch = parse_line.get_char();
        if ch == '*' && next_ch == '/' {
            return true;
        }
        ch = next_ch;
    }
    false
}

/// 数字文字列のトークン化
/// ただし有効な数字かどうかはチェックしない
fn get_number(parse_line: &mut LineParser, rawtoken_chars: &mut Vec<char>) -> Result<(), ()> {
    while parse_line.has_char() {
        let ch = parse_line.peek_char();
        // 記号またはスペースの場合はトークン確定
        match ch {
            ' ' | symbols_without_dot_or_space!() => {
                return Ok(());
            }
            _ => {}
        }
        rawtoken_chars.push(ch);
        if ch.is_ascii_alphanumeric() || ch == '.' {
            parse_line.proceed();
        } else {
            return Err(());
        }
    }
    Ok(())
}

// 識別子のトークン化
fn get_identifier(parse_line: &mut LineParser, rawtoken_chars: &mut Vec<char>) -> Result<(), ()> {
    while parse_line.has_char() {
        let ch = parse_line.peek_char();
        // 記号またはスペースの場合はトークン確定
        match ch {
            ' ' | '.' | symbols_without_dot_or_space!() => {
                return Ok(());
            }
            _ => {}
        }
        rawtoken_chars.push(ch);
        if ch.is_ascii_alphanumeric() || ch == '_' {
            parse_line.proceed();
        } else {
            return Err(());
        }
    }
    Ok(())
}

// 複数記号トークンとなるか確認する
fn get_mult_symbol(parse_line: &mut LineParser, rawtoken_chars: &mut Vec<char>) {
    let mut rawtoken_string = rawtoken_chars.iter().collect::<String>();
    match parse_line.peek_nextchar(0) {
        Some(ch) => {
            rawtoken_string.push(ch);
        }
        None => {
            return;
        }
    }
    let mut found_towchars_symbol = false;
    for symbol in twochars_symbol_array!() {
        if rawtoken_string == symbol {
            found_towchars_symbol = true;
            break;
        }
    }
    if !found_towchars_symbol {
        return;
    }

    parse_line.proceed();
    rawtoken_chars.push(rawtoken_string.chars().nth(1).unwrap());
    match parse_line.peek_nextchar(0) {
        Some(ch) => {
            rawtoken_string.push(ch);
        }
        None => {
            return;
        }
    }
    let mut found_threechars_symbol = false;
    for symbol in threechars_symbol_array!() {
        if rawtoken_string == symbol {
            found_threechars_symbol = true;
            break;
        }
    }
    if !found_threechars_symbol {
        return;
    }
    rawtoken_chars.push(rawtoken_string.chars().nth(2).unwrap());
    parse_line.proceed();
}

// 記号トークン作成
// <<のような複数記号のトークンも作成する(<トークン2つにしない)
fn get_symbol(parse_line: &mut LineParser, token_chars: &mut Vec<char>) {
    get_mult_symbol(parse_line, token_chars);
}

fn get_quote_text(parse_line: &mut LineParser, rawtoken_chars: &mut Vec<char>) -> Result<bool, ()> {
    let head_ch = rawtoken_chars[0];
    let mut last_ch = parse_line.peek_char();
    while parse_line.has_char() {
        let ch = parse_line.get_char();
        rawtoken_chars.push(ch);
        if head_ch == ch && *rawtoken_chars.last().unwrap() != '\\' {
            return Ok(true);
        }
        last_ch = ch;
    }
    if last_ch == '\\' {
        Ok(false)
    } else {
        Err(())
    }
}

pub fn parse_file(filepath: &Path) -> Vec<RawToken> {
    let file;
    if let Ok(fd) = File::open(filepath) {
        file = fd;
    } else {
        eprintln!("error: no such file {}", filepath.to_str().unwrap());
        std::process::exit(-1);
    }

    let reader = BufReader::new(file);
    let mut rawtokens: Vec<RawToken> = vec![];
    let mut rawtoken_chars: Vec<char> = vec![];
    let mut parser = Parser::new();

    let source_txt: &mut Vec<String>;
    unsafe {
        SOURCE_TXT.clear();
        source_txt = &mut SOURCE_TXT;
    }

    for (line_num, line) in reader.lines().enumerate() {
        let line_txt = line.unwrap();
        let mut parse_line = LineParser::new(line_txt.chars().collect());
        source_txt.push(line_txt);
        while parse_line.has_char() {
            match parser.state {
                ParserState::Empty => {
                    rawtoken_chars.clear();
                    parser.line = line_num;
                    parser.pos = parse_line.get_pos();
                    match initialize_parser(&mut parse_line, &mut rawtoken_chars) {
                        Ok(state) => {
                            parser.state = state;
                        }
                        Err(()) => {
                            let err_token = rawtoken_chars.iter().collect::<String>();
                            exit_parser_error(ParserError::InvalidIdentifiler(err_token), &parser);
                        }
                    }
                }
                ParserState::LineComment => {
                    parser.state = ParserState::Empty;
                    break;
                }
                ParserState::Comment => {
                    if proceed_until_comment_closed(&mut parse_line) {
                        parser.state = ParserState::Empty;
                    }
                }
                ParserState::Number => match get_number(&mut parse_line, &mut rawtoken_chars) {
                    Ok(()) => {
                        let rawtoken = RawToken::new_number_rawtoken(&rawtoken_chars, &parser);
                        rawtokens.push(rawtoken);
                        parser.state = ParserState::Empty;
                    }
                    Err(()) => {
                        let err_token = rawtoken_chars.iter().collect::<String>();
                        exit_parser_error(ParserError::InvalidIdentifiler(err_token), &parser);
                    }
                },
                ParserState::QuoteText => {
                    match get_quote_text(&mut parse_line, &mut rawtoken_chars) {
                        Ok(closed) => {
                            if closed {
                                let rawtoken = RawToken::new(
                                    &rawtoken_chars,
                                    &parser,
                                    RawTokenKind::QuoteText,
                                );
                                rawtokens.push(rawtoken);
                                parser.state = ParserState::Empty;
                            }
                        }
                        Err(()) => {
                            exit_parser_error(ParserError::UnClosedError, &parser);
                        }
                    }
                }
                ParserState::Symbol => {
                    get_symbol(&mut parse_line, &mut rawtoken_chars);
                    let rawtoken = RawToken::new(&rawtoken_chars, &parser, RawTokenKind::Symbol);
                    rawtokens.push(rawtoken);
                    parser.state = ParserState::Empty;
                }
                ParserState::Identifier => {
                    match get_identifier(&mut parse_line, &mut rawtoken_chars) {
                        Ok(()) => {
                            let token =
                                RawToken::new(&rawtoken_chars, &parser, RawTokenKind::Identifier);
                            rawtokens.push(token);
                            parser.state = ParserState::Empty;
                        }
                        Err(()) => {
                            let err_token = rawtoken_chars.iter().collect::<String>();
                            exit_parser_error(ParserError::InvalidIdentifiler(err_token), &parser);
                        }
                    }
                }
            }
        }
        // 行末まで到達
        // initialize_tokenでトークンの種類を確定させて行末に来た場合はここで識別子, 記号, 文字列トークン確定を行う
        // それ以外のケースではEmptyになっているか複数行トークン継続中
        match parser.state {
            ParserState::Identifier => {
                let rawtoken = RawToken::new(&rawtoken_chars, &parser, RawTokenKind::Identifier);
                rawtokens.push(rawtoken);
                parser.state = ParserState::Empty;
            }
            ParserState::Number => {
                let rawtoken = RawToken::new_number_rawtoken(&rawtoken_chars, &parser);
                rawtokens.push(rawtoken);
                parser.state = ParserState::Empty;
            }
            ParserState::Symbol => {
                let rawtoken = RawToken::new(&rawtoken_chars, &parser, RawTokenKind::Symbol);
                rawtokens.push(rawtoken);
                parser.state = ParserState::Empty;
            }
            ParserState::LineComment => {
                parser.state = ParserState::Empty;
            }
            _ => {}
        }
    }

    // ファイル端で未トークン化があればエラーとする
    if parser.state != ParserState::Empty {
        exit_parser_error(ParserError::UnClosedError, &parser);
    }

    // トークンが1つもない場合はエラーとする
    if rawtokens.is_empty() {
        exit_no_token_err();
    }
    rawtokens
}
