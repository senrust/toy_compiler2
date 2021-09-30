pub mod tokenizer;

use tokenizer::{tokenize};

fn main() {
    let tokens = tokenize("main.c");
    for token in tokens {
        println!("{}", token.token);
    }
}
