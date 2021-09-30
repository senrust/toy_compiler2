use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::exit;
use std::fmt;

/// トークン情報
/// * type - トークンのタイプ
/// * line - トークンの行番号
/// * pos - トークンの列番号
pub struct Token {
    pub token: String,
    pub line: usize,
    pub pos: usize,
}

impl Token {
    fn new(token: &Vec<char>, info: &TokenizeInfo) -> Token {
        Token {
            token: token.iter().collect(), 
            line: info.line,
            pos: info.pos,
        }
    }
}

fn tokenizer_panic(err: TokenizeError, info: TokenizeInfo) -> ! {
    eprintln!("line{}, pos{}, error: {}", info.line, info.pos, err);
    exit(-1);
}

enum TokenizeError {
    InvalidIdentifiler(String),
    UnClosedError,
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenizeError::InvalidIdentifiler(identifier) => {
                write!(f, "{} is invalid identifier", identifier)
            },
            TokenizeError::UnClosedError => {
                write!(f, "not closed")
            },
        }
    }
}

#[derive(PartialEq, Eq)]
enum TokenState {
    Empty,
    Comment,
    LineComment,
    Identifier,
    QuoteText,
    Number,
    Symbol,
}

struct TokenizeInfo {
    state: TokenState,
    line: usize,
    pos: usize,
}

impl TokenizeInfo {
    fn new() -> Self {
        TokenizeInfo {
            state: TokenState::Empty,
            line: 0,
            pos:0,
        }
    }
}

struct TokenizeLine {
    line: Vec<char>,
    cur: usize,
}

impl TokenizeLine {
    fn new(line: Vec<char>) -> Self {
        TokenizeLine {
            line,
            cur: 0,
        }
    }

    fn get_pos(&self) -> usize {
        self.cur + 1
    }

    fn has_char(&self) -> bool {
        if self.cur < self.line.len() {
            true
        } else {
            false
        }
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

fn initialize_token(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) -> Result<TokenState, ()> {
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
        |'+'|'-'|'/'|'*'|'='|'.'
        |'&'|'^'|'<'|'>'|'|'
        |'('|')'|'['|']'|'{'|'}'|';' => {
            return Ok(TokenState::Symbol);
        }
        _ => {},
    }

    if ch.is_ascii_alphabetic() {
        return Ok(TokenState::Identifier);
    }

    if ch == '"' || ch == '\'' {
        return Ok(TokenState::QuoteText);
    }

    Err(())
}

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
            |'+'|'-'|'/'|'*'|'='|'.'
            |'&'|'^'|'<'|'>'|'|'
            |'('|')'|'['|']'|'{'|'}'|';' => {
                return Ok(());
            }
        _ => {},
        }
        token_chars.push(ch);
        if ch.is_ascii_alphanumeric() {
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
            |' '
            |'+'|'-'|'/'|'*'|'='|'.'
            |'&'|'^'|'<'|'>'|'|'
            |'('|')'|'['|']'|'{'|'}'|';' => {
                return Ok(());
            }
        _ => {},
        }
        token_chars.push(ch);
        if ch.is_ascii_alphanumeric() {
            tokenize_line.proceed();
        } else {
            return Err(());
        }
    }
    Ok(())
}

fn is_mult_symbol(ch: char, next_ch: char, mult_symbols: &[&str]) -> bool {
    for mult_symbol in mult_symbols {
        let mut mult_symbol_chars = mult_symbol.chars();
        if ch == mult_symbol_chars.next().unwrap() && next_ch == mult_symbol_chars.next().unwrap() {
            return true;
        }
    }
    false
}

fn get_symbol(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) {
    if tokenize_line.has_char() {
        let ch = token_chars[0];
        let next_ch = tokenize_line.peek_char();
        let mult_symbols = ["<<", ">>", "++", "--", "==", "||", "&&"];
        if is_mult_symbol(ch, next_ch, &mult_symbols) {
            token_chars.push(next_ch);
            tokenize_line.proceed();
        }
        
    }
}

fn get_quote_text(tokenize_line: &mut TokenizeLine, token_chars: &mut Vec<char>) -> Result<bool, ()> {
    let head_ch = token_chars[0];
    let mut last_ch = tokenize_line.peek_char();
    while tokenize_line.has_char() {
        let ch = tokenize_line.get_char();
        token_chars.push(ch);
        if head_ch == ch  && *token_chars.last().unwrap() != '\\' {
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

pub fn tokenize(filepath: &str) -> Vec<Token> {
    let file = File::open(filepath).unwrap();
    let reader = BufReader::new(file);

    let mut tokens: Vec<Token> = vec![];
    let mut token_chars: Vec<char> = vec![];
    let mut tokenize_state = TokenizeInfo::new();

    for (line_num, line) in reader.lines().enumerate() {
        let mut tokenizeline = TokenizeLine::new(line.unwrap().chars().collect());
        while tokenizeline.has_char() {
            match tokenize_state.state {
                TokenState::Empty => {
                    token_chars.clear();
                    tokenize_state.line = line_num + 1;
                    tokenize_state.pos = tokenizeline.get_pos();
                    match initialize_token(&mut tokenizeline, &mut token_chars) {
                        Ok(state) => {
                            tokenize_state.state = state;
                        }
                        Err(()) => {
                            let err_token = token_chars.iter().collect::<String>();
                            tokenizer_panic(TokenizeError::InvalidIdentifiler(err_token), tokenize_state);
                        }
                    }

                },
                TokenState::LineComment => {
                    tokenize_state.state = TokenState::Empty;
                    break;
                }
                TokenState::Comment => {
                    if proceed_until_comment_closed(&mut tokenizeline) {
                        tokenize_state.state = TokenState::Empty;
                    }
                }
                TokenState::Number => {
                    match get_number(&mut tokenizeline, &mut token_chars) {
                        Ok(()) => {
                            let token = Token::new(&token_chars, &tokenize_state);
                            tokens.push(token);
                            tokenize_state.state = TokenState::Empty;
                        }
                        Err(()) => {
                            let err_token = token_chars.iter().collect::<String>();
                            tokenizer_panic(TokenizeError::InvalidIdentifiler(err_token), tokenize_state);
                        }
                    }
                }
                TokenState::QuoteText => {
                    match get_quote_text(&mut tokenizeline, &mut token_chars) {
                        Ok(closed) => {
                            if closed {
                                let token = Token::new(&token_chars, &tokenize_state);
                                tokens.push(token);
                                tokenize_state.state = TokenState::Empty;
                            }
                        },
                        Err(()) => {
                            tokenizer_panic(TokenizeError::UnClosedError, tokenize_state);
                        }
                    }
                }
                TokenState::Symbol => {
                    get_symbol(&mut tokenizeline, &mut token_chars);
                    let token = Token::new(&token_chars, &tokenize_state);
                    tokens.push(token);
                    tokenize_state.state = TokenState::Empty;

                }
                TokenState::Identifier => {
                    match get_identifier(&mut tokenizeline, &mut token_chars) {
                        Ok(()) => {
                            let token = Token::new(&token_chars, &tokenize_state);
                            tokens.push(token);
                            tokenize_state.state = TokenState::Empty;
                        }
                        Err(()) => {
                            let err_token = token_chars.iter().collect::<String>();
                            tokenizer_panic(TokenizeError::InvalidIdentifiler(err_token), tokenize_state);
                        }
                    }
                }
            }
        }
        // 行末まで到達
        // initialize_tokenでトークンの種類を確定させて行末に来た場合はここで識別子, 記号, 文字列トークン確定を行う
        // それ以外はEmptyに戻している
        // ただし複数行コメント, 複数行文字列のトークン確定は行わない
        match tokenize_state.state {
            TokenState::Identifier | TokenState::Symbol | TokenState::Number => {
                let token = Token::new(&token_chars, &tokenize_state);
                tokens.push(token);
                tokenize_state.state = TokenState::Empty;
            }
            TokenState::LineComment => {
                tokenize_state.state = TokenState::Empty;
            }
            _ => {

            }
        }
    }

    // ファイル端で未トークン化があればエラーとする
    if tokenize_state.state != TokenState::Empty {
        tokenizer_panic(TokenizeError::UnClosedError, tokenize_state);
    }
    tokens
}