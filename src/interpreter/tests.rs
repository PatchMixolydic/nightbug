use crate::{
    interpreter::{Binding, Interpreter},
    lexer::lex,
    parser::{parse, ExprKind}
};

macro_rules! interpret_str {
    ($s:literal) => {{
        let exprs = parse(lex($s).unwrap(), $s).unwrap();
        Interpreter::new().interpret(exprs.into_iter()).unwrap()
    }};
}

macro_rules! assert_result_expr {
    ($code:literal, $kind:expr) => {
        match interpret_str!($code) {
            Binding::Expression(expr) => assert_eq!(expr.kind, $kind),
            binding => panic!("Result of {} is not an expression: {:?}", $code, binding)
        }
    };
}

macro_rules! assert_result_matches {
    ($code:literal, $expect:pat) => {
        assert!(matches!(interpret_str!($code), $expect))
    };
}

#[test]
fn integer_literal() {
    assert_result_expr!("2", ExprKind::Integer(2));
}

#[test]
fn boolean_literal() {
    assert_result_expr!("true", ExprKind::Boolean(true));
    assert_result_expr!("false", ExprKind::Boolean(false));
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
    assert_result_expr!("(add 2 3)", ExprKind::Integer(5));
}

#[test]
fn second() {
    assert_result_expr!("(second 2 3)", ExprKind::Integer(3));
}

#[test]
fn composed_add_second() {
    assert_result_expr!("(add 2 (second 3 4))", ExprKind::Integer(6));
}
