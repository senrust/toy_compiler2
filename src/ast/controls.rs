use crate::ast::ast::*;
use crate::ast::operations::*;
use crate::definition::definitions::Definitions;
use crate::definition::reservedwords::*;
use crate::definition::symbols::*;
use crate::definition::types::Type;
use crate::token::error::*;
use crate::token::token::Tokens;

// return = "return" assign
// return は returnする対象をもつ
pub fn ast_return(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // consume "return"
    let info = tokens.consume_reserved(Reserved::Return);
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
    let mut if_ast_vec: Vec<Option<Ast>> = vec![];
    // consume "if"
    let if_info = tokens.consume_reserved(Reserved::If);
    let if_type = definitions.get_type("void").unwrap();

    // consume "("
    tokens.consume_symbol(Symbol::LeftParenthesis);
    let condition_ast = ast_assign(tokens, definitions);
    // consume ")"
    tokens.consume_symbol(Symbol::RightParenthesis);
    // true時のAst
    let true_ast = ast_expr(tokens, definitions);
    if_ast_vec.push(Some(true_ast));
    if tokens.expect_reserved(Reserved::Else) {
        // consume "else"
        tokens.consume_reserved(Reserved::Else);
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
    // consume "if"
    let for_info = tokens.consume_reserved(Reserved::For);
    let for_type = definitions.get_type("void").unwrap();
    let mut for_contitions: Vec<Option<Ast>> = vec![];

    tokens.consume_symbol(Symbol::LeftParenthesis); // consume "("

    // ローカル変数のスコープを深くする
    definitions.enter_new_local_scope();

    for i in 0..3 {
        if tokens.expect_symbol(Symbol::SemiColon) {
            for_contitions.push(None);
        } else {
            let inilaize_ast = ast_assign(tokens, definitions);
            for_contitions.push(Some(inilaize_ast));
        }
        if i != 2 {
            tokens.consume_symbol(Symbol::SemiColon); // consume ";"
        }
    }
    tokens.consume_symbol(Symbol::RightParenthesis); // consume ")"

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

// while = "while"  "(" assign ")" expr
// whileは条件をcontextへ, exprs[0]にwhile内容を格納
pub fn ast_while(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // consume "while"
    let while_info = tokens.consume_reserved(Reserved::While);
    let while_type = definitions.get_type("void").unwrap();
    let mut while_vec: Vec<Ast> = vec![];

    tokens.consume_symbol(Symbol::LeftParenthesis); // consume "("
    let while_condition = ast_assign(tokens, definitions);

    tokens.consume_symbol(Symbol::RightParenthesis); // consume ")"
    let while_expr = ast_expr(tokens, definitions);
    while_vec.push(while_expr);
    Ast::new_control_ast(
        while_info,
        while_type,
        Control::While,
        Some(Box::new(while_condition)),
        Some(while_vec),
        None,
    )
}

// break
// breakして脱出するラベルはコンパイラ側で決定する
pub fn ast_break(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    // consume "break"
    let break_info = tokens.consume_reserved(Reserved::Break);
    let break_type = definitions.get_type("void").unwrap();
    Ast::new_control_ast(break_info, break_type, Control::Break, None, None, None)
}

fn ast_function_args(
    args_types: &Option<Vec<Type>>,
    tokens: &mut Tokens,
    definitions: &mut Definitions,
) -> Option<Vec<Ast>> {
    // 引数ありの場合
    if let Some(arg_types) = args_types {
        let mut args_ast: Vec<Ast> = vec![];
        tokens.consume_symbol(Symbol::LeftParenthesis); // consume "("
        for (count, arg_type) in arg_types.iter().enumerate() {
            //let type_;
            let ast;
            if tokens.expect_number() {
                ast = ast_number(tokens, definitions);
            } else if tokens.expect_identifier() {
                ast = ast_variable(tokens, definitions);
            } else {
                output_unclosed_token_err(tokens);
            }
            // 現在は不要だが方のチェックを行う
            if ast.type_ != *arg_type {
                output_defferenttype_err(tokens);
            }
            args_ast.push(ast);
            if count == arg_types.len() - 1 {
                break;
            } else {
                tokens.consume_symbol(Symbol::Comma);
            }
        }

        if !tokens.expect_symbol(Symbol::RightParenthesis) {
            output_incorrectarg_err(tokens);
        }
        tokens.consume(); // consume ")"
        Some(args_ast)
    } else {
        tokens.consume_symbol(Symbol::LeftParenthesis); // consume "("
        if !tokens.expect_symbol(Symbol::RightParenthesis) {
            output_incorrectarg_err(tokens);
        }
        tokens.consume(); // consume ")"
        None
    }
}

// functioncall = funcname "(" args ")"
pub fn ast_functioncall(tokens: &mut Tokens, definitions: &mut Definitions) -> Ast {
    let (funcname, info) = tokens.consume_identifier();
    if let Some(type_) = definitions.get_type(&funcname) {
        let func = type_.function.as_ref().unwrap();
        // 返り値の型を関数の型とする
        let ret_type;
        if let Some(ret) = &func.ret {
            ret_type = ret.clone();
        } else {
            ret_type = definitions.get_type("void").unwrap();
        }
        // 引数を設定
        let args = ast_function_args(&func.args, tokens, definitions);
        Ast::new_functioncall_ast(&funcname, info, type_, ret_type, args)
    } else {
        output_undefinedfunction_err(&info);
    }
}
