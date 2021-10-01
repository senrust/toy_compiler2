pub mod tokenizer;

use tokenizer::tokenize;

static mut SOURCE_TXT: Vec<String> = vec![];
fn main() {
    let tokens = tokenize("main.c");
    for token in tokens {
        println!("{}", token.token);
    }
}
