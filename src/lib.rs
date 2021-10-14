#![allow(dead_code)]
#![allow(clippy::result_unit_err, clippy::module_inception)]
pub mod ast;
mod definition;
pub mod output;
pub mod token;

static mut SOURCE_TXT: Vec<String> = vec![];
