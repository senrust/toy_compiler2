use crate::definition::number::{string_to_number, Number};
use crate::definition::reservedwords::*;
use crate::definition::symbols::{get_token_symbol, Symbol};
use crate::definition::types::{PrimitiveType, PrimitiveTypeError};
use crate::token::parser::{RawToken, RawTokenKind};
use std::fmt;

#[derive(Debug)]
pub enum TokenKind {
    Number(String),
    Symbol(Symbol),
    Identifier(String),
    Reserved(Reserved),
    RawString(String),
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub line: usize,
    pub pos: usize,
    pub width: usize,
}

impl TokenInfo {
    pub fn new(line: usize, pos: usize, width: usize) -> Self {
        TokenInfo { line, pos, width }
    }
}

fn get_token_kind(rawtoken: RawToken) -> (TokenKind, TokenInfo) {
    let info = TokenInfo::new(rawtoken.line, rawtoken.pos, rawtoken.rawtoken.len());
    match rawtoken.kind {
        RawTokenKind::Number => (TokenKind::Number(rawtoken.rawtoken), info),
        RawTokenKind::Identifier => {
            if let Some(reserved) = check_reserved_word(&rawtoken.rawtoken) {
                (TokenKind::Reserved(reserved), info)
            } else {
                (TokenKind::Identifier(rawtoken.rawtoken), info)
            }
        }
        RawTokenKind::QuoteText => (TokenKind::RawString(rawtoken.rawtoken), info),
        RawTokenKind::Symbol => {
            let symbol = get_token_symbol(rawtoken.rawtoken);
            (TokenKind::Symbol(symbol), info)
        }
    }
}

pub struct Token {
    pub info: TokenInfo,
    pub kind: TokenKind,
}

impl Token {
    fn new(rawtoken: RawToken) -> Self {
        let (kind, info) = get_token_kind(rawtoken);
        Token { info, kind }
    }

    pub fn expect_symbol(&self, expected_symbol: &Symbol) -> bool {
        match self.kind {
            TokenKind::Symbol(ref symbol) => *symbol == *expected_symbol,
            _ => false,
        }
    }

    pub fn expect_number(&self) -> bool {
        matches!(self.kind, TokenKind::Number(_))
    }

    pub fn get_interger(&self) -> Result<Number, ()> {
        match self.kind {
            TokenKind::Number(ref num_txt) => {
                if let Ok(num) = string_to_number(num_txt) {
                    Ok(num)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }

    pub fn expect_identifier(&self) -> bool {
        matches!(self.kind, TokenKind::Identifier(_))
    }

    pub fn get_identifier(&self) -> Result<&String, ()> {
        match self.kind {
            TokenKind::Identifier(ref identifier) => Ok(identifier),
            _ => Err(()),
        }
    }

    pub fn expect_rawstring(&self) -> bool {
        matches!(self.kind, TokenKind::RawString(_))
    }

    pub fn get_rawstring(&self) -> Option<&String> {
        match self.kind {
            TokenKind::RawString(ref rawstring) => Some(rawstring),
            _ => None,
        }
    }

    pub fn expect_reserved(&self, reserved: Reserved) -> bool {
        match self.kind {
            TokenKind::Reserved(ref word) => *word == reserved,
            _ => false,
        }
    }

    pub fn expect_primitivetype(&self) -> bool {
        match &self.kind {
            TokenKind::Reserved(reserved) => check_primitivetype_reserved_word(reserved),
            _ => false,
        }
    }

    pub fn get_primitivetypename(&self) -> Result<PrimitiveType, PrimitiveTypeError> {
        match &self.kind {
            TokenKind::Reserved(reserved) => get_primitivetype_reserved_word(reserved),
            _ => Err(PrimitiveTypeError::NotPrimitiveTypeErr),
        }
    }
}

pub enum TokenError {
    UnexpectTokenError,
    InvalidNumberErr(String),
    UnClosedError,
    UnClosedEndError,
    UnDeclaredVariableError,
    AlreadyDeclaredVariableError,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenError::UnexpectTokenError => {
                write!(f, "unexpected token")
            }
            TokenError::InvalidNumberErr(str) => {
                write!(f, "{} is invalid number", str)
            }
            TokenError::UnClosedError => {
                write!(f, "unclosed")
            }
            TokenError::UnClosedEndError => {
                write!(f, "unclosed end")
            }
            TokenError::UnDeclaredVariableError => {
                write!(f, "undeclared variable")
            }
            TokenError::AlreadyDeclaredVariableError => {
                write!(f, "already declared variable")
            }
        }
    }
}

pub fn make_tokens(rawtokens: Vec<RawToken>) -> Tokens {
    let mut tokens: Vec<Token> = vec![];
    for rawtoken in rawtokens {
        let token = Token::new(rawtoken);
        tokens.push(token);
    }
    Tokens::new(tokens)
}

pub struct Tokens {
    vec: Vec<Token>,
    cur: usize,
}

impl Tokens {
    pub fn new(token_vec: Vec<Token>) -> Self {
        Tokens {
            vec: token_vec,
            cur: 0,
        }
    }

    pub fn get(&self) -> Option<&Token> {
        self.vec.get(self.cur)
    }

    pub fn get_tail(&self) -> Option<&Token> {
        self.vec.last()
    }

    pub fn is_empty(&self) -> bool {
        !self.has_token()
    }

    pub fn has_token(&self) -> bool {
        self.cur < self.vec.len()
    }

    pub fn consume(&mut self) -> Result<TokenInfo, ()> {
        if let Some(token) = self.vec.get(self.cur) {
            self.cur += 1;
            Ok(token.info.clone())
        } else {
            Err(())
        }
    }

    pub fn expect_symbol(&self, symbol: Symbol) -> bool {
        if let Some(token) = self.vec.get(self.cur) {
            if token.expect_symbol(&symbol) {
                return true;
            }
        }
        false
    }

    pub fn expect_symbols(&self, symbols: &[Symbol]) -> bool {
        for symbol in symbols {
            if let Some(token) = self.vec.get(self.cur) {
                if token.expect_symbol(symbol) {
                    return true;
                }
            }
        }
        false
    }

    pub fn expect_number(&self) -> bool {
        if let Some(token) = self.vec.get(self.cur) {
            token.expect_number()
        } else {
            false
        }
    }

    pub fn consume_integer(&mut self) -> Result<(Number, TokenInfo), ()> {
        if let Some(token) = self.vec.get(self.cur) {
            if let Ok(num) = token.get_interger() {
                self.cur += 1;
                Ok((num, token.info.clone()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn expect_identifier(&mut self) -> bool {
        if let Some(token) = self.vec.get(self.cur) {
            token.expect_identifier()
        } else {
            false
        }
    }

    pub fn consume_identifier(&mut self) -> Result<(String, TokenInfo), ()> {
        if let Some(token) = self.vec.get(self.cur) {
            if let Ok(ident) = token.get_identifier() {
                self.cur += 1;
                Ok((ident.clone(), token.info.clone()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn expect_reserved(&self, reserved: Reserved) -> bool {
        if let Some(token) = self.vec.get(self.cur) {
            token.expect_reserved(reserved)
        } else {
            false
        }
    }

    pub fn expect_primitivetype(&self) -> bool {
        if let Some(token) = self.vec.get(self.cur) {
            if token.expect_primitivetype() {
                return true;
            }
        }
        false
    }

    pub fn consume_primitivetype(&mut self) -> Result<PrimitiveType, ()> {
        if let Some(token) = self.vec.get(self.cur) {
            let primitive_type = token.get_primitivetypename();
            match primitive_type {
                Ok(type_) => {
                    self.cur += 1;
                    Ok(type_)
                }
                Err(PrimitiveTypeError::UnsignedError) => {
                    self.cur += 1;
                    if let Some(token) = self.vec.get(self.cur) {
                        if let Ok(primitive_type) = token.get_primitivetypename() {
                            self.cur += 1;
                            let type_ = match primitive_type {
                                PrimitiveType::I8 => PrimitiveType::U8,
                                PrimitiveType::I16 => PrimitiveType::U16,
                                PrimitiveType::I32 => PrimitiveType::U32,
                                PrimitiveType::I64 => PrimitiveType::U64,
                                _ => return Err(()),
                            };
                            self.cur += 1;
                            Ok(type_)
                        } else {
                            Err(())
                        }
                    } else {
                        Err(())
                    }
                }
                Err(PrimitiveTypeError::NotPrimitiveTypeErr) => Err(()),
            }
        } else {
            Err(())
        }
    }
}
