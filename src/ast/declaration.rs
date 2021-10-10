use crate::definition::definitions::Definitions;
use crate::definition::symbols::Symbol;
use crate::token::error::*;
use crate::token::token::Tokens;

pub fn local_val_declaration(tokens: &mut Tokens, definitions: &mut Definitions) {
    if !tokens.expect_primitivetype() {
        output_unexpected_token_err(tokens);
    }

    let primitive_type = tokens.consume_primitivetype().unwrap();
    let type_ = definitions.get_primitive_type(&primitive_type);
    if let Ok((name, info)) = tokens.consume_identifier() {
        let declare_sucess = definitions.declar_local_val(&name, type_).is_ok();
        if declare_sucess {
            if tokens.expect_symbol(Symbol::SemiColon) {
                // consume ";"
                tokens.consume().unwrap();
            } else {
                output_unexpected_token_err(tokens);
            }
        } else {
            output_alreadydeclared_variable_err(&info);
        }
    } else {
        output_unexpected_token_err(tokens);
    }
}
