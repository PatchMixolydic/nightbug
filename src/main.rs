#![feature(or_patterns)]

pub mod lexer;
pub mod parser;

fn main() {
    println!("{:?}", parser::parse(lexer::lex("(add 2 (add 2 2))")));
}
