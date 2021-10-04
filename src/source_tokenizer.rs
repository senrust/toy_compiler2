use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::tokenizer_error;
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
            "<<", ">>", "++", "--", "==", "||", "&&", "+=", "-=", "*=", "/=", "%=", "&=", "^=",
            "|=", "->",
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
pub enum TokenKind {
    Identifier,
    Number,
    Symbol,
    QuoteText,
}
/// トークン情報
/// * token - トークン文字列
/// * kind - トークン種別
/// * line - トークンの行番号
/// * pos - トークンの列番号
pub struct Token {
    pub token: String,
    pub kind: TokenKind,
    pub line: usize,
    pub pos: usize,
}

impl Token {
    fn new(token: &Vec<char>, info: &TokenizeInfo, kind: TokenKind) -> Token {
        Token {
            token: token.iter().collect(),
            kind,
            line: info.line,
            pos: info.pos,
        }
    }

    fn new_number_token(token: &Vec<char>, info: &TokenizeInfo) -> Token {
        let token_string: String = token.iter().collect();
        Token {
            token: token_string,
            kind: TokenKind::Number,
            line: info.line,
            pos: info.pos,
        }
    }
}

pub enum TokenizeError {
    InvalidIdentifiler(String),
    UnClosedError,
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenizeError::InvalidIdentifiler(identifier) => {
                write!(f, "{} is invalid identifier", identifier)
            }
            TokenizeError::UnClosedError => {
                write!(f, "not closed")
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum TokenState {
    Empty,
    Comment,
    LineComment,
    Identifier,
    QuoteText,
    Number,
    Symbol,
}

pub struct TokenizeInfo {
    pub state: TokenState,
    pub line: usize,
    pub pos: usize,
}

impl TokenizeInfo {
    fn new() -> Self {
        TokenizeInfo {
            state: TokenState::Empty,
            line: 0,
            pos: 0,
        }
    }
}

struct TokenizeLine {
    line: Vec<char>,
    cur: usize,
}

impl TokenizeLine {
    fn new(line: Vec<char>) -> Self {
        TokenizeLine { line, cur: 0 }
    }

    fn get_pos(&self) -> usize {
        self.cur
    }

    fn has_char(&self) -> bool {
        if self.cur < self.line.len() {
            true
        } else {
            false
        }
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
fn initialize_token(
    tokenize_line: &mut TokenizeLine,
    token_chars: &mut Vec<char>,
) -> Result<TokenState, ()> {
    // 1行コメントチェック
    if tokenize_line.compare_text("//") {
        tokenize_line.advance(2);
        return Ok(TokenState::LineComment);
    }

    // 複数行コメントチェック
    if tokenize_line.compare_text("/*") {
        tokenize_line.advance(2);
        return Ok(TokenState::Comment);
    }

    let ch = tokenize_line.get_char();
    if ch == ' ' {
        return Ok(TokenState::Empty);
    }

    token_chars.push(ch);
    if ch.is_ascii_digit() {
        return Ok(TokenState::Number);
    }

    match ch {
        symbols_without_dot_or_space!() | '.' => {
            return Ok(TokenState::Symbol);
        }
        _ => {}
    }

    if ch.is_ascii_alphabetic() || ch == '_' {
        return Ok(TokenState::Identifier);
    }

    if ch == '"' || ch == '\'' {
        return Ok(TokenState::QuoteText);
    }

    Err(())
}

// コメントが閉じられるまでループ
// コメントが閉じられない場合は複数行コメントとして次の行で閉じられるまで再ループを行う
fn proceed_until_comment_closed(tokenize_line: &mut TokenizeLine) -> bool {
    let mut ch = tokenize_line.get_char();
    while tokenize_line.has_char() {
        let next_ch = tokenize_line.get_char();
        if ch == '*' && next_ch == '/' {
            return true;
        }
        ch = next_ch;
    }
    false
}

/// 数字文字列のトークン化
/// ただし有効な数字かどうかはチェックしない
fn get_number(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) -> Result<(), ()> {
    while tokenize_line.has_char() {
        let ch = tokenize_line.peek_char();
        // 記号またはスペースの場合はトークン確定
        match ch {
            ' ' | symbols_without_dot_or_space!() => {
                return Ok(());
            }
            _ => {}
        }
        token_chars.push(ch);
        if ch.is_ascii_alphanumeric() || ch == '.' {
            tokenize_line.proceed();
        } else {
            return Err(());
        }
    }
    Ok(())
}

// 識別子のトークン化
fn get_identifier(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) -> Result<(), ()> {
    while tokenize_line.has_char() {
        let ch = tokenize_line.peek_char();
        // 記号またはスペースの場合はトークン確定
        match ch {
            ' ' | '.' | symbols_without_dot_or_space!() => {
                return Ok(());
            }
            _ => {}
        }
        token_chars.push(ch);
        if ch.is_ascii_alphanumeric() || ch == '_' {
            tokenize_line.proceed();
        } else {
            return Err(());
        }
    }
    Ok(())
}

// 複数記号トークンとなるか確認する
fn get_mult_symbol(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) {
    let mut current_token = token_chars.iter().collect::<String>();
    match tokenize_line.peek_nextchar(0) {
        Some(ch) => {
            current_token.push(ch);
        }
        None => {
            return;
        }
    }
    let mut found_towchars_symbol = false;
    for symbol in twochars_symbol_array!() {
        if current_token == symbol {
            found_towchars_symbol = true;
            break;
        }
    }
    if !found_towchars_symbol {
        return;
    }

    tokenize_line.proceed();
    token_chars.push(current_token.chars().nth(1).unwrap());
    match tokenize_line.peek_nextchar(0) {
        Some(ch) => {
            current_token.push(ch);
        }
        None => {
            return;
        }
    }
    let mut found_threechars_symbol = false;
    for symbol in threechars_symbol_array!() {
        if current_token == symbol {
            found_threechars_symbol = true;
            break;
        }
    }
    if !found_threechars_symbol {
        return;
    }
    token_chars.push(current_token.chars().nth(2).unwrap());
    tokenize_line.proceed();
}

// 記号トークン作成
// <<のような複数記号のトークンも作成する(<トークン2つにしない)
fn get_symbol(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) {
    get_mult_symbol(tokenize_line, token_chars);
}

fn get_quote_text(
    tokenize_line: &mut TokenizeLine,
    token_chars: &mut Vec<char>,
) -> Result<bool, ()> {
    let head_ch = token_chars[0];
    let mut last_ch = tokenize_line.peek_char();
    while tokenize_line.has_char() {
        let ch = tokenize_line.get_char();
        token_chars.push(ch);
        if head_ch == ch && *token_chars.last().unwrap() != '\\' {
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

pub fn tokenize(filepath: &Path) -> Vec<Token> {
    let file = File::open(filepath).unwrap();
    let reader = BufReader::new(file);

    let mut tokens: Vec<Token> = vec![];
    let mut token_chars: Vec<char> = vec![];
    let mut tokenize_state = TokenizeInfo::new();

    let source_txt: &mut Vec<String>;
    unsafe {
        source_txt = &mut SOURCE_TXT;
    }

    for (line_num, line) in reader.lines().enumerate() {
        let line_txt = line.unwrap();
        let mut tokenizeline = TokenizeLine::new(line_txt.chars().collect());
        source_txt.push(line_txt);
        while tokenizeline.has_char() {
            match tokenize_state.state {
                TokenState::Empty => {
                    token_chars.clear();
                    tokenize_state.line = line_num;
                    tokenize_state.pos = tokenizeline.get_pos();
                    match initialize_token(&mut tokenizeline, &mut token_chars) {
                        Ok(state) => {
                            tokenize_state.state = state;
                        }
                        Err(()) => {
                            let err_token = token_chars.iter().collect::<String>();
                            tokenizer_error(
                                TokenizeError::InvalidIdentifiler(err_token),
                                &tokenize_state,
                            );
                        }
                    }
                }
                TokenState::LineComment => {
                    tokenize_state.state = TokenState::Empty;
                    break;
                }
                TokenState::Comment => {
                    if proceed_until_comment_closed(&mut tokenizeline) {
                        tokenize_state.state = TokenState::Empty;
                    }
                }
                TokenState::Number => match get_number(&mut tokenizeline, &mut token_chars) {
                    Ok(()) => {
                        let token = Token::new_number_token(&token_chars, &tokenize_state);
                        tokens.push(token);
                        tokenize_state.state = TokenState::Empty;
                    }
                    Err(()) => {
                        let err_token = token_chars.iter().collect::<String>();
                        tokenizer_error(
                            TokenizeError::InvalidIdentifiler(err_token),
                            &tokenize_state,
                        );
                    }
                },
                TokenState::QuoteText => {
                    match get_quote_text(&mut tokenizeline, &mut token_chars) {
                        Ok(closed) => {
                            if closed {
                                let token =
                                    Token::new(&token_chars, &tokenize_state, TokenKind::QuoteText);
                                tokens.push(token);
                                tokenize_state.state = TokenState::Empty;
                            }
                        }
                        Err(()) => {
                            tokenizer_error(TokenizeError::UnClosedError, &tokenize_state);
                        }
                    }
                }
                TokenState::Symbol => {
                    get_symbol(&mut tokenizeline, &mut token_chars);
                    let token = Token::new(&token_chars, &tokenize_state, TokenKind::Symbol);
                    tokens.push(token);
                    tokenize_state.state = TokenState::Empty;
                }
                TokenState::Identifier => {
                    match get_identifier(&mut tokenizeline, &mut token_chars) {
                        Ok(()) => {
                            let token =
                                Token::new(&token_chars, &tokenize_state, TokenKind::Identifier);
                            tokens.push(token);
                            tokenize_state.state = TokenState::Empty;
                        }
                        Err(()) => {
                            let err_token = token_chars.iter().collect::<String>();
                            tokenizer_error(
                                TokenizeError::InvalidIdentifiler(err_token),
                                &tokenize_state,
                            );
                        }
                    }
                }
            }
        }
        // 行末まで到達
        // initialize_tokenでトークンの種類を確定させて行末に来た場合はここで識別子, 記号, 文字列トークン確定を行う
        // それ以外のケースではEmptyになっているか複数行トークン継続中
        match tokenize_state.state {
            TokenState::Identifier => {
                let token = Token::new(&token_chars, &tokenize_state, TokenKind::Identifier);
                tokens.push(token);
                tokenize_state.state = TokenState::Empty;
            }
            TokenState::Number => {
                let token = Token::new_number_token(&token_chars, &tokenize_state);
                tokens.push(token);
                tokenize_state.state = TokenState::Empty;
            }
            TokenState::Symbol => {
                let token = Token::new(&token_chars, &tokenize_state, TokenKind::Symbol);
                tokens.push(token);
                tokenize_state.state = TokenState::Empty;
            }
            TokenState::LineComment => {
                tokenize_state.state = TokenState::Empty;
            }
            _ => {}
        }
    }

    // ファイル端で未トークン化があればエラーとする
    if tokenize_state.state != TokenState::Empty {
        tokenizer_error(TokenizeError::UnClosedError, &tokenize_state);
    }
    tokens
}
