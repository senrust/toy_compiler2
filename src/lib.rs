mod definition;
mod error;
pub mod interpret_token; 
pub mod make_ast;
pub mod tokenizer;

static mut SOURCE_TXT: Vec<String> = vec![];