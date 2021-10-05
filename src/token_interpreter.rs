use crate::definition::number::{string_to_number, Number};
use crate::definition::symbols::{get_token_symbol, Symbol};
use crate::source_tokenizer::{Token, TokenKind};
use std::fmt;

#[derive(Debug)]
pub enum NodeKind {
    Number(String),
    Symbol(Symbol),
    Identifier(String),
    RawString(String),
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub line: usize,
    pub pos: usize,
    pub width: usize,
}

impl NodeInfo {
    pub fn new(line: usize, pos: usize, width: usize) -> Self {
        NodeInfo { line, pos, width }
    }
}

fn get_node_kind(token: Token) -> (NodeKind, NodeInfo) {
    let info = NodeInfo::new(token.line, token.pos, token.token.len());
    match token.kind {
        TokenKind::Number => {
            return (NodeKind::Number(token.token), info);
        }
        TokenKind::Identifier => {
            return (NodeKind::Identifier(token.token), info);
        }
        TokenKind::QuoteText => {
            return (NodeKind::RawString(token.token), info);
        }
        TokenKind::Symbol => {
            let symbol = get_token_symbol(token.token);
            return (NodeKind::Symbol(symbol), info);
        }
    }
}

pub struct Node {
    pub info: NodeInfo,
    pub kind: NodeKind,
}

impl Node {
    fn new(token: Token) -> Self {
        let (kind, info) = get_node_kind(token);
        Node { info, kind }
    }

    pub fn expect_symbol(&self, expected_symbol: &Symbol) -> bool {
        match self.kind {
            NodeKind::Symbol(ref symbol) => {
                return *symbol == *expected_symbol;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn expect_number(&self) -> bool {
        match self.kind {
            NodeKind::Number(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn get_interger(&self) -> Result<Number, ()> {
        match self.kind {
            NodeKind::Number(ref num_txt) => {
                if let Ok(num) = string_to_number(num_txt) {
                    return Ok(num);
                } else {
                    return Err(());
                }
            }
            _ => {
                return Err(());
            }
        }
    }

    pub fn expect_identifier(&self) -> bool {
        match self.kind {
            NodeKind::Identifier(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn get_identifier(&self) -> Result<&String, ()> {
        match self.kind {
            NodeKind::Identifier(ref identifier) => {
                return Ok(identifier);
            }
            _ => {
                return Err(());
            }
        }
    }

    pub fn expect_rawstring(&self) -> bool {
        match self.kind {
            NodeKind::RawString(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn get_rawstring(&self) -> Option<&String> {
        match self.kind {
            NodeKind::RawString(ref rawstring) => {
                return Some(rawstring);
            }
            _ => {
                return None;
            }
        }
    }
}

pub enum NodeError {
    NotValueError,
    UnexpectNodeError,
    UnexpectEndError,
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeError::NotValueError => {
                write!(f, "value is expected")
            }
            NodeError::UnexpectNodeError => {
                write!(f, "unexpected token")
            }
            NodeError::UnexpectEndError => {
                write!(f, "unexpected end")
            }
        }
    }
}

pub fn make_nodes(tokens: Vec<Token>) -> Nodes {
    let mut nodes: Vec<Node> = vec![];
    for token in tokens {
        let node = Node::new(token);
        nodes.push(node);
    }
    Nodes::new(nodes)
}

pub struct Nodes {
    vec: Vec<Node>,
    cur: usize,
}

impl Nodes {
    pub fn new(node_vec: Vec<Node>) -> Self {
        Nodes {
            vec: node_vec,
            cur: 0,
        }
    }

    pub fn get(&self) -> Option<&Node> {
        self.vec.get(self.cur)
    }

    pub fn get_last(&self) -> Option<&Node> {
        self.vec.last()
    }

    pub fn proceed(&mut self) {
        self.cur += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.cur >= self.vec.len()
    }

    pub fn has_node(&self) -> bool {
        self.cur < self.vec.len()
    }

    pub fn consume(&mut self) -> Result<NodeInfo, ()> {
        if let Some(node) = self.vec.get(self.cur) {
            self.cur += 1;
            Ok(node.info.clone())
        } else {
            Err(())
        }
    }

    pub fn expect_symbol(&mut self, symbol: Symbol) -> bool {
        if let Some(node) = self.vec.get(self.cur) {
            if node.expect_symbol(&symbol) {
                return true;
            }
        }
        false
    }

    pub fn expect_symbols(&mut self, symbols: &[Symbol]) -> bool {
        for symbol in symbols {
            if let Some(node) = self.vec.get(self.cur) {
                if node.expect_symbol(symbol) {
                    return true;
                }
            }
        }
        false
    }

    pub fn expect_number(&self) -> bool {
        if let Some(node) = self.vec.get(self.cur) {
            node.expect_number()
        } else {
            false
        }
    }

    pub fn consume_integer(&mut self) -> Result<(Number, NodeInfo), ()> {
        if let Some(node) = self.vec.get(self.cur) {
            if let Ok(num) = node.get_interger() {
                self.cur += 1;
                Ok((num, node.info.clone()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}
