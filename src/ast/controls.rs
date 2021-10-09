use crate::ast::ast::*;
use crate::definition::definitions::Definitions;
use crate::definition::reservedwords::*;
use crate::definition::symbols::*;
use crate::token::error::*;
use crate::token::token::Tokens;

// return = "return" assign
// return は returnする対象をもつ
pub fn ast_return(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_reserved(Reserved::Return) {
        output_unexpected_token_err(tokens);
    }
    // consume "return"
    let info = tokens.consume().unwrap();
    let return_value = ast_assign(tokens, definitions);
    let type_ = return_value.type_.clone();
    // 今後関数の定義されている戻り型と比較を行う
    // 即;ならばvoid型に設定する
    let context = vec![return_value];
    Ast::new_control_ast(info, type_, Control::Return, None, Some(context), None)
}

// if = "if" "(" assign ")" expr ("else" expr)?
// if は contextに条件式, exprs[0]に trueのAst, exprs[1]にfalseのAstが入る
pub fn ast_if(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_reserved(Reserved::If) {
        output_unexpected_token_err(tokens);
    }

    let mut if_ast_vec: Vec<Option<Ast>> = vec![];
    // consume "if"
    let if_info = tokens.consume().unwrap();
    let if_type = definitions.get_type("void").unwrap();
    if !tokens.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // consume "("
    tokens.consume().unwrap();
    let condition_ast = ast_assign(tokens, definitions);
    if !tokens.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_token_err(tokens);
    }
    // consume ")"
    tokens.consume().unwrap();
    // true時のAst
    let true_ast = ast_expr(tokens, definitions);
    if_ast_vec.push(Some(true_ast));
    if tokens.expect_reserved(Reserved::Else) {
        // consume "else"
        tokens.consume().unwrap();
        let else_ast = ast_expr(tokens, definitions);
        if_ast_vec.push(Some(else_ast));
    } else {
        if_ast_vec.push(None);
    }
    Ast::new_control_ast(
        if_info,
        if_type,
        Control::If,
        Some(Box::new(condition_ast)),
        None,
        Some(if_ast_vec),
    )
}

// "for" "(" expr? ";" expr? ";" expr? ")" expr
pub fn ast_for(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    if !tokens.expect_reserved(Reserved::For) {
        output_unexpected_token_err(tokens);
    }

    // consume "if"
    let for_info = tokens.consume().unwrap();
    let for_type = definitions.get_type("void").unwrap();
    let mut for_contitions: Vec<Option<Ast>> = vec![];
    if !tokens.expect_symbol(Symbol::LeftParenthesis) {
        output_unexpected_token_err(tokens);
    }
    tokens.consume().unwrap(); // consume "("

    // ローカル変数のスコープを深くする
    definitions.enter_new_local_scope();

    for i in 0..3 {
        if tokens.expect_symbol(Symbol::SemiColon) {
            for_contitions.push(None);
        } else {
            let inilaize_ast = ast_expr(tokens, definitions);
            for_contitions.push(Some(inilaize_ast));
        }
        if i != 2 {
            if !tokens.expect_symbol(Symbol::SemiColon) {
                output_unexpected_token_err(tokens);
            }
            tokens.consume().unwrap(); // consume ";"
        }
    }
    if !tokens.expect_symbol(Symbol::RightParenthesis) {
        output_unexpected_token_err(tokens);
    }
    tokens.consume().unwrap(); // consume ")"

    let for_context = ast_expr(tokens, definitions);
    // ローカル変数のスコープから出る
    definitions.exit_current_local_scope();

    Ast::new_control_ast(
        for_info,
        for_type,
        Control::For,
        Some(Box::new(for_context)),
        None,
        Some(for_contitions),
    )
}
