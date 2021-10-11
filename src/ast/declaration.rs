use crate::definition::definitions::Definitions;
use crate::definition::symbols::Symbol;
use crate::definition::types::Type;
use crate::token::error::*;
use crate::token::token::Tokens;

pub fn is_type_token(tokens: &mut Tokens, _definitions: &mut Definitions) -> bool {
    if tokens.expect_primitivetype() {
        true
    } else {
        false
    }
}

pub fn cousume_type_token(tokens: &mut Tokens, definitions: &mut Definitions) -> Type {
    if let Ok(primitive_type) = tokens.get_primitivetype() {
        definitions.get_primitive_type(&primitive_type)
    } else {
        output_unexpected_token_err(tokens);
    }
}

pub fn local_val_declaration(tokens: &mut Tokens, definitions: &mut Definitions) {
    let type_ = cousume_type_token(tokens, definitions);
    let (name, info) = tokens.consume_identifier();
    let declare_sucess = definitions.declare_local_val(&name, type_).is_ok();
    if declare_sucess {
        tokens.consume_symbol(Symbol::SemiColon);
    } else {
        output_alreadydeclared_variable_err(&info);
    }
}
