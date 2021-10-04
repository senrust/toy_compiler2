pub mod ast_maker;
mod definition;
mod error;
pub mod source_tokenizer;
pub mod token_interpreter;

static mut SOURCE_TXT: Vec<String> = vec![];
