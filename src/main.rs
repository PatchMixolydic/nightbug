#![feature(once_cell)]
#![feature(or_patterns)]
#![feature(type_alias_impl_trait)]

pub mod interpreter;
pub mod lexer;
pub mod parser;

fn main() {
    let code = "(add 2 (second 3 4))";
    println!("Code: {:?}", code);
    let tokens = lexer::lex(code);
    println!("Tokens: {:?}", tokens);
    let expressions = parser::parse(tokens);
    println!("Expressions: {:?}", expressions);
    println!("Result: {:?}", interpreter::interpret(expressions.into_iter()));
}
