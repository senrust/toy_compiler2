use crate::tokenizer::{NumberToken, Token, TokenKind};

#[derive(Debug)]
pub enum NumberKind {
    U64(u64),
    Double(f64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Period,
    Comma,
    Colon,
    SemiColon,
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitNot,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    Not,
    And,
    Or,
    Eq,
    NotEq,
    Gt,
    Lt,
    Ge,
    Le,
    Increment,
    Decrement,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    LeftShiftAssign,
    RightShiftAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    Reference,
    Sharp,
}

fn get_symbol_token_kind(token: String) -> SymbolKind {
    match token.as_str() {
        "." => SymbolKind::Period,
        "," => SymbolKind::Comma,
        ":" => SymbolKind::Colon,
        ";" => SymbolKind::SemiColon,
        "=" => SymbolKind::Assign,
        "+" => SymbolKind::Add,
        "-" => SymbolKind::Sub,
        "*" => SymbolKind::Mul,
        "/" => SymbolKind::Div,
        "%" => SymbolKind::Rem,
        "~" => SymbolKind::BitNot,
        "&" => SymbolKind::BitAnd,
        "|" => SymbolKind::BitOr,
        "^" => SymbolKind::BitXor,
        "<<" => SymbolKind::LeftShift,
        ">>" => SymbolKind::RightShift,
        "!" => SymbolKind::Not,
        "&&" => SymbolKind::And,
        "||" => SymbolKind::Or,
        "==" => SymbolKind::Eq,
        "!=" => SymbolKind::NotEq,
        "<" => SymbolKind::Gt,
        ">" => SymbolKind::Lt,
        "<=" => SymbolKind::Ge,
        ">=" => SymbolKind::Le,
        "++" => SymbolKind::Increment,
        "--" => SymbolKind::Decrement,
        "+=" => SymbolKind::AddAssign,
        "-=" => SymbolKind::SubAssign,
        "*=" => SymbolKind::MulAssign,
        "/=" => SymbolKind::DivAssign,
        "%=" => SymbolKind::RemAssign,
        "<<=" => SymbolKind::LeftShiftAssign,
        ">>=" => SymbolKind::RightShiftAssign,
        "&=" => SymbolKind::AndAssign,
        "|=" => SymbolKind::OrAssign,
        "^=" => SymbolKind::XorAssign,
        "(" => SymbolKind::LeftParenthesis,
        ")" => SymbolKind::RightParenthesis,
        "[" => SymbolKind::LeftSquareBracket,
        "]" => SymbolKind::RightSquareBracket,
        "{" => SymbolKind::LeftCurlyBracket,
        "}" => SymbolKind::RightCurlyBracket,
        "->" => SymbolKind::Reference,
        "#" => SymbolKind::Sharp,
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Number(NumberKind),
    Symbol(SymbolKind),
    Identifier(String),
    RawString(String),
}

#[derive(Debug)]

pub struct NodeInfo {
    line: usize,
    pos: usize,
    width: usize,
}

impl NodeInfo {
    fn new(line: usize, pos: usize, width: usize) -> Self {
        NodeInfo { line, pos, width }
    }
}

fn get_node_kind(token: Token) -> (NodeKind, NodeInfo) {
    let info = NodeInfo::new(token.line, token.pos, token.token.len());
    match token.kind {
        TokenKind::Number(ref num_token) => match num_token {
            NumberToken::U64(num) => {
                return (NodeKind::Number(NumberKind::U64(*num)), info);
            }
            NumberToken::Double(num) => {
                return (NodeKind::Number(NumberKind::Double(*num)), info);
            }
        },
        TokenKind::Identifier => {
            return (NodeKind::Identifier(token.token), info);
        }
        TokenKind::QuoteText => {
            return (NodeKind::RawString(token.token), info);
        }
        TokenKind::Symbol => {
            let symbol_kind = get_symbol_token_kind(token.token);
            return (NodeKind::Symbol(symbol_kind), info);
        }
    }
}

pub struct Node {
    pub info: NodeInfo,
    pub kind: NodeKind,
}

impl Node {
    fn new(token: Token) -> Self {
        let (node_kind, node_info) = get_node_kind(token);
        Node {
            info: node_info,
            kind: node_kind,
        }
    }

    fn expect_symbol(&self, symbol_kind: SymbolKind) -> bool {
        match self.kind {
            NodeKind::Symbol(ref symbol) => {
                return *symbol == symbol_kind;
            }
            _ => {
                return false;
            }
        }
    }

    fn expect_num(&self) -> bool {
        match self.kind {
            NodeKind::Number(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn get_interger(&self) -> Option<u64> {
        match self.kind {
            NodeKind::Number(ref num) => match num {
                NumberKind::U64(integer) => {
                    return Some(*integer);
                }
                NumberKind::Double(_float) => {
                    return None;
                }
            },
            _ => {
                return None;
            }
        }
    }

    fn get_float(&self) -> Option<f64> {
        match self.kind {
            NodeKind::Number(ref num) => match num {
                NumberKind::U64(_integer) => {
                    return None;
                }
                NumberKind::Double(float) => {
                    return Some(*float);
                }
            },
            _ => {
                return None;
            }
        }
    }

    fn expect_identifier(&self) -> bool {
        match self.kind {
            NodeKind::Identifier(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn get_identifier(&self) -> Option<&String> {
        match self.kind {
            NodeKind::Identifier(ref identifier) => {
                return Some(identifier);
            }
            _ => {
                return None;
            }
        }
    }

    fn expect_rawstring(&self) -> bool {
        match self.kind {
            NodeKind::RawString(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn get_rawstring(&self) -> Option<&String> {
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

pub fn make_nodes(tokens: Vec<Token>) -> Vec<Node> {
    let mut nodes: Vec<Node> = vec![];
    for token in tokens {
        let node = Node::new(token);
        nodes.push(node);
    }
    nodes
}
