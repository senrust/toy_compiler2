use crate::definition::definitions::Definitions;
use crate::definition::symbols::Symbol;
use crate::definition::types::Type;
use crate::token::error::*;
use crate::token::token::{TokenInfo, TokenKind, Tokens};

pub fn is_type_token(tokens: &mut Tokens, definitions: &mut Definitions) -> bool {
    // 現在はプリミティブ型のみ対応
    if tokens.expect_primitivetype() {
        true
    } else if tokens.expect_identifier() {
        let token = tokens.get().unwrap();
        if let TokenKind::Identifier(name) = &token.kind {
            matches!(definitions.get_type(name), Some(_))
        } else {
            false
        }
    } else {
        false
    }
}

// 型, 変数名, 変数名トークン位置を返す
// 関数宣言のみ変数名指定が不要なので, その場合の変数名は空文字列, トークン位置は変数名が期待される位置の直前とする
// (関数宣言時のトークン位置は使用しないので問題ない)
pub fn cousume_type_token(
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> (Type, String, TokenInfo) {
    // 現在はプリミティブ型のみ対応
    let mut type_: Type;
    if let Ok(primitive_type) = tokens.get_primitivetype() {
        type_ = definitions.get_primitive_type(&primitive_type);
    } else {
        output_unexpected_token_err(tokens);
    }
    // ポインター型
    loop {
        if tokens.expect_symbol(Symbol::Mul) {
            tokens.consume_symbol(Symbol::Mul);
            type_ = Type::new_pointer(type_);
        } else {
            break;
        }
    }
    // 変数名を取得
    let valname: String;
    let info: TokenInfo;
    if tokens.expect_identifier() {
        let tmp = tokens.consume_identifier();
        valname = tmp.0;
        info = tmp.1;
    } else {
        valname = "".to_string();
        info = tokens.get_prev(1).unwrap().info;
    }

    // 配列型か判定 n次元配列に対応するためループ
    let mut array_size_vec: Vec<usize> = vec![];
    while tokens.expect_symbol(Symbol::LeftSquareBracket) {
        tokens.consume_symbol(Symbol::LeftSquareBracket);
        let (elem_num, info) = tokens.consume_integer();
        if let Ok(elem_count) = elem_num.get_usize_value() {
            array_size_vec.push(elem_count);
        } else {
            output_notinteger_err(&info);
        }
        tokens.consume_symbol(Symbol::RightSquareBracket);
    }

    for elem_count in array_size_vec.iter().rev() {
        type_ = Type::new_array(*elem_count, type_);
    }
    (type_, valname, info)
}

pub fn local_val_declaration(tokens: &mut Tokens, definitions: &mut Definitions) {
    let (type_, name, info) = cousume_type_token(tokens, definitions);
    let declare_sucess = definitions.declare_local_val(&name, type_).is_ok();
    if declare_sucess {
        tokens.consume_symbol(Symbol::SemiColon);
    } else {
        output_alreadydeclared_variable_err(&info);
    }
}
