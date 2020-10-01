use crate::{
    interpreter::{interpret, Binding},
    lexer::lex,
    parser::{parse, Expr}
};

macro_rules! interpret_str {
    ($s:literal) => {
        interpret(parse(lex($s).unwrap(), $s).unwrap().into_iter())
    };
}

macro_rules! assert_result {
    ($code:literal, $expect:expr) => {
        assert_eq!(interpret_str!($code), $expect)
    };
}

macro_rules! assert_result_matches {
    ($code:literal, $expect:pat) => {
        assert!(matches!(interpret_str!($code), $expect))
    };
}

const fn bind(expr: Expr) -> Binding {
    Binding::Expression(expr)
}

#[test]
fn integer_literal() {
    assert_result!("2", bind(Expr::Integer(2)));
}

#[test]
fn boolean_literal() {
    assert_result!("true", bind(Expr::Boolean(true)));
    assert_result!("false", bind(Expr::Boolean(false)));
}

#[test]
fn nightbug_function() {
    // TODO: define function here
    assert_result_matches!("second", Binding::Function(..))
}

#[test]
fn native_function() {
    assert_result_matches!("add", Binding::NativeFunction(..))
}

#[test]
fn add() {
    assert_result!("(add 2 3)", bind(Expr::Integer(5)));
}

#[test]
fn second() {
    assert_result!("(second 2 3)", bind(Expr::Integer(3)));
}

#[test]
fn composed_add_second() {
    assert_result!("(add 2 (second 3 4))", bind(Expr::Integer(6)));
}
