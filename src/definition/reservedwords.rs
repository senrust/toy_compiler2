macro_rules! reserved_words_array {
    () => {
        [
            "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
            "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
            "return", "signed", "sizeof", "short", "static", "struct", "switch", "typedef",
            "union", "unsigned", "void", "volatile", "while",
        ]
    };
}

#[derive(Clone, Debug)]
pub enum Reserved {
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Signed,
    Sizeof,
    Short,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

fn get_reserved_word_type(indentifiler: &str) -> Reserved {
    match indentifiler {
        "auto" => Reserved::Auto,
        "break" => Reserved::Break,
        "case" => Reserved::Case,
        "char" => Reserved::Char,
        "const" => Reserved::Const,
        "continue" => Reserved::Continue,
        "default" => Reserved::Default,
        "do" => Reserved::Do,
        "double" => Reserved::Double,
        "else" => Reserved::Else,
        "enum" => Reserved::Enum,
        "extern" => Reserved::Extern,
        "float" => Reserved::Float,
        "for" => Reserved::For,
        "goto" => Reserved:: Goto,
        "if" => Reserved::If,
        "int" => Reserved::Int,
        "long" => Reserved::Long,
        "register" => Reserved::Register,
        "return" => Reserved::Return,
        "signed" => Reserved::Signed,
        "sizeof" => Reserved::Sizeof,
        "short" => Reserved::Short,
        "static" => Reserved::Static,
        "struct" => Reserved::Struct,
        "switch" => Reserved::Switch,
        "typedef" => Reserved::Typedef,
        "union" => Reserved::Union,
        "unsigned" => Reserved::Unsigned,
        "void" => Reserved::Void,
        "volatile" => Reserved::Volatile,
        "while" => Reserved::While,
        _ => unreachable!(),
    }
}

pub fn check_reserved_word(indentifiler: &str) -> Option<Reserved> {
    for reserved_words in reserved_words_array!() {
        if indentifiler == reserved_words {
            return Some(get_reserved_word_type(indentifiler));
        }
    }
    None
}
