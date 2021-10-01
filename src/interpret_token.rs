use crate::tokenizer::{NumberToken, Token, TokenKind};

#[derive(Debug)]
pub enum NumberKind {
    U32(u32),
    Double(f64),
}

#[derive(Debug)]
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
}

fn get_symbol_token_kind(token: &Token) -> SymbolKind {
    match token.token.as_str() {
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
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum ASTNodeKind {
    Number(NumberKind),
    Symbol(SymbolKind),
    Identifier(String),
    RawString(String),
}

#[derive(Debug)]

pub struct ASTNodeInfo {
    line: usize,
    pos: usize,
    width: usize,
}

impl ASTNodeInfo {
    fn new(line: usize, pos: usize, width: usize) -> Self {
        ASTNodeInfo { line, pos, width }
    }
}

fn get_node_kind(tokens: &mut Tokens) -> (ASTNodeKind, ASTNodeInfo) {
    let token = tokens.get_token();
    let info = ASTNodeInfo::new(token.line, token.pos, token.token.len());
    match token.kind {
        TokenKind::Number(ref num_token) => match num_token {
            NumberToken::U32(num) => {
                return (ASTNodeKind::Number(NumberKind::U32(*num)), info);
            }
            NumberToken::Double(num) => {
                return (ASTNodeKind::Number(NumberKind::Double(*num)), info);
            }
        },
        TokenKind::Identifier => {
            return (ASTNodeKind::Identifier(token.token.clone()), info);
        }
        TokenKind::QuoteText => {
            return (ASTNodeKind::RawString(token.token.clone()), info);
        }
        TokenKind::Symbol => {
            let symbol_kind = get_symbol_token_kind(token);
            return (ASTNodeKind::Symbol(symbol_kind), info);
        }
    }
}

pub struct ASTNode {
    pub info: ASTNodeInfo,
    pub kind: ASTNodeKind,
}

impl ASTNode {
    fn new(tokens: &mut Tokens) -> Self {
        let (node_kind, node_info) = get_node_kind(tokens);
        ASTNode {
            info: node_info,
            kind: node_kind,
        }
    }
}

struct Tokens {
    vec: Vec<Token>,
    cur: usize,
}

impl Tokens {
    fn has_token(&self) -> bool {
        if self.cur < self.vec.len() {
            true
        } else {
            false
        }
    }

    fn get_token(&mut self) -> &Token {
        let token = &self.vec[self.cur];
        self.cur += 1;
        token
    }
}

pub fn make_ast_nodes(token_vec: Vec<Token>) -> Vec<ASTNode> {
    let mut tokens = Tokens {
        vec: token_vec,
        cur: 0,
    };
    let mut ast_nodes: Vec<ASTNode> = vec![];
    while tokens.has_token() {
        let node = ASTNode::new(&mut tokens);
        ast_nodes.push(node);
    }
    ast_nodes
}
