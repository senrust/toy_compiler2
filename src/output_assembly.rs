use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ast_maker::*;

fn write_assembly_header<T: Write>(buf: &mut T) {
    writeln!(buf, ".intel_syntax noprefix").unwrap();
    writeln!(buf, ".globl main").unwrap();
    writeln!(buf, "").unwrap();
}

pub fn output_assembly(_asts: Vec<AST>, output_file: &Path) {
    let mut buf = BufWriter::new(fs::File::create(output_file).unwrap());
    write_assembly_header(&mut buf);
    writeln!(buf, "main:").unwrap();
    writeln!(buf, "    push 10").unwrap();
    writeln!(buf, "    pop rax").unwrap();
    writeln!(buf, "    ret").unwrap();
}
