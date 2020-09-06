use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::parser::Expr;

static INTERPRETER_STATE: SyncLazy<HashMap<String, Binding>> = SyncLazy::new(|| {
    let mut res = HashMap::new();
    res.insert("add".to_string(), Binding::NativeFunction(None, add_native));
    res.insert("second".to_string(), Binding::Function(2, Expr::Argument(1)));
    res
});

type Expressions = std::vec::IntoIter<Expr>;
type Bindings = std::vec::IntoIter<Binding>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Binding {
    Expression(Expr),
    Function(usize /* num_arguments */, Expr),
    NativeFunction(Option<usize>, fn(Bindings) -> Binding)
}

pub fn interpret(mut expressions: Expressions) -> Binding {
    let expression = match expressions.next() {
        Some(expr) => expr,
        None => return Binding::Expression(Expr::Unit)
    };

    match expression {
        Expr::List(inner_expressions) => interpret(inner_expressions.into_iter()),
        Expr::Identifier(ident) => handle_identifier(&ident, expressions),
        expr @ (Expr::Integer(_) | Expr::Boolean(_) | Expr::Unit) => Binding::Expression(expr),
        Expr::Keyword(_) => todo!(),
        Expr::Argument(_) => unreachable!()
    }
}

fn handle_identifier(ident: &str, expressions: Expressions) -> Binding {
    let binding = INTERPRETER_STATE.get(ident)
        .expect(&format!("Unknown identifier {}", ident));

    match binding {
        Binding::Expression(expr) => return Binding::Expression(expr.clone()),

        func @ (Binding::Function(..) | Binding::NativeFunction(..)) => {
            if expressions.len() != 0 {
                return handle_function(func, &ident, expressions);
            } else {
                return func.clone();
            }
        },
    }
}

fn handle_function(func: &Binding, ident: &str, expressions: Expressions) -> Binding {
    assert!(matches!(func, Binding::Function(..) | Binding::NativeFunction(..)));

    match func {
        Binding::Function(num_arguments, body) => {
            if expressions.len() != *num_arguments {
                panic!(
                    "Wrong number of arguments for function {} (expected {}, got {})",
                    ident,
                    num_arguments,
                    expressions.len()
                )
            }

            let mut body = body.clone();
            let args: Vec<Expr> = expressions.collect();
        
            if let Expr::Argument(num) = body {
                body = args[num].clone();
            }

            match body {
                Expr::List(contents) => {
                    let contents: Vec<Expr> = contents
                        .into_iter()
                        .map(|expr| {
                            match expr {
                                Expr::Argument(num) => args[num].clone(),
                                _ => expr,
                            }
                        })
                        // Collection needed to avoid recursion limit
                        .collect();
                    interpret(contents.into_iter())
                },

                Expr::Identifier(_) => handle_identifier(ident, Vec::new().into_iter()),
                expr @ (Expr::Integer(_) | Expr::Boolean(_) | Expr::Unit) => Binding::Expression(expr),
                _ => todo!()
            }
        },

        Binding::NativeFunction(maybe_num_arguments, func) => {
            if let Some(num_arguments) = maybe_num_arguments {
                if expressions.len() != *num_arguments {
                    panic!(
                        "Wrong number of arguments for function {} (expected {}, got {})",
                        ident,
                        maybe_num_arguments.unwrap(),
                        expressions.len()
                    )
                }
            }

            let bindings = expressions
                .into_iter()
                .map(|expr| {
                    if let Expr::List(contents) = expr {
                        interpret(contents.into_iter())
                    } else {
                        Binding::Expression(expr)
                    }
                })
                // Apply the map
                .collect::<Vec<Binding>>()
                // We also need an IntoIter
                .into_iter();

            func(bindings)
        },

        _ => unreachable!()
    }
}

fn add_native(bindings: Bindings) -> Binding {
    let res = bindings
        .into_iter()
        .fold(0, |acc, next| {
            if let Binding::Expression(Expr::Integer(i)) = next {
                acc + i
            } else {
                panic!("Unexpected argument to add: {:?}", next)
            }
        });

    Binding::Expression(Expr::Integer(res))
}
