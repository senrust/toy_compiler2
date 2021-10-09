#![allow(dead_code)]
pub mod ast;
mod definition;
pub mod output;
pub mod token;

static mut SOURCE_TXT: Vec<String> = vec![];
