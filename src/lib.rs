#![allow(dead_code)]
pub mod ast;
pub mod token;
mod definition;
pub mod output;

static mut SOURCE_TXT: Vec<String> = vec![];
