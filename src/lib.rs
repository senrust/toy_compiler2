mod definition;
mod error;
pub mod token_interpreter; 
pub mod ast_maker;
pub mod tokenizer;

static mut SOURCE_TXT: Vec<String> = vec![];