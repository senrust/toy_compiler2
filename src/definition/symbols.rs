#[derive(Debug, PartialEq, Eq)]
pub enum Symbol {
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
    LeftParenthesis,    // (
    RightParenthesis,   // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    LeftCurlyBracket,   // {
    RightCurlyBracket,  // }
    Reference,
    Sharp,
}

pub fn get_token_symbol(token: String) -> Symbol {
    match token.as_str() {
        "." => Symbol::Period,
        "," => Symbol::Comma,
        ":" => Symbol::Colon,
        ";" => Symbol::SemiColon,
        "=" => Symbol::Assign,
        "+" => Symbol::Add,
        "-" => Symbol::Sub,
        "*" => Symbol::Mul,
        "/" => Symbol::Div,
        "%" => Symbol::Rem,
        "~" => Symbol::BitNot,
        "&" => Symbol::BitAnd,
        "|" => Symbol::BitOr,
        "^" => Symbol::BitXor,
        "<<" => Symbol::LeftShift,
        ">>" => Symbol::RightShift,
        "!" => Symbol::Not,
        "&&" => Symbol::And,
        "||" => Symbol::Or,
        "==" => Symbol::Eq,
        "!=" => Symbol::NotEq,
        "<" => Symbol::Gt,
        ">" => Symbol::Lt,
        "<=" => Symbol::Ge,
        ">=" => Symbol::Le,
        "++" => Symbol::Increment,
        "--" => Symbol::Decrement,
        "+=" => Symbol::AddAssign,
        "-=" => Symbol::SubAssign,
        "*=" => Symbol::MulAssign,
        "/=" => Symbol::DivAssign,
        "%=" => Symbol::RemAssign,
        "<<=" => Symbol::LeftShiftAssign,
        ">>=" => Symbol::RightShiftAssign,
        "&=" => Symbol::AndAssign,
        "|=" => Symbol::OrAssign,
        "^=" => Symbol::XorAssign,
        "(" => Symbol::LeftParenthesis,
        ")" => Symbol::RightParenthesis,
        "[" => Symbol::LeftSquareBracket,
        "]" => Symbol::RightSquareBracket,
        "{" => Symbol::LeftCurlyBracket,
        "}" => Symbol::RightCurlyBracket,
        "->" => Symbol::Reference,
        "#" => Symbol::Sharp,
        _ => unreachable!(),
    }
}
