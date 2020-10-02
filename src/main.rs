//! Nightbug is a Lisp-like programming language.
//! Please note that it is a very early work in progress,
//! so the language is likely highly unstable and probably
//! inefficient. Please be careful!

pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod parser;

fn main() {
    let code = "(add 2 (second 3 4))";
    println!("Code: {:?}", code);
    println!();

    let tokens = match lexer::lex(code) {
        Ok(res) => res,
        Err(_) => return
    };
    println!("Tokens: {:?}", tokens);
    println!();

    let expressions = match parser::parse(tokens, code) {
        Ok(res) => res,
        Err(_) => return
    };
    println!("Expressions: {:?}", expressions);
    println!();

    let mut interpreter = interpreter::Interpreter::new();
    println!(
        "Result: {:?}",
        interpreter.interpret_with_source(expressions, code)
    );
}
