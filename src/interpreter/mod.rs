#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::parser::Expr;

type Expressions = std::vec::IntoIter<Expr>;
type Bindings = std::vec::IntoIter<Binding>;

/// The value of a binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Binding {
    /// An expression (ex. "(add 2 2)")
    Expression(Expr),
    /// A function defined in Nightbug
    Function(usize /* num_arguments */, Expr),
    /// A function defined in Rust
    NativeFunction(Option<usize>, fn(Bindings) -> Binding)
}

pub struct Interpreter {
    bindings: HashMap<String, Binding>
}

impl Interpreter {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("add".to_string(), Binding::NativeFunction(None, add_native));
        bindings.insert(
            "second".to_string(),
            Binding::Function(2, Expr::Argument(1))
        );

        Self { bindings }
    }

    /// Interpret a given iterator over expressions.
    /// Returns a binding as the result of the evaluation.
    pub fn interpret(&mut self, mut expressions: Expressions) -> Binding {
        let expression = match expressions.next() {
            Some(expr) => expr,
            None => return Binding::Expression(Expr::Unit)
        };

        match expression {
            Expr::List(inner_expressions) => self.interpret(inner_expressions.into_iter()),
            Expr::Identifier(ident) => self.handle_identifier(&ident, expressions),
            Expr::Integer(_) | Expr::Boolean(_) | Expr::Unit => Binding::Expression(expression),
            Expr::Keyword(_) => todo!(),
            Expr::Argument(_) => unreachable!()
        }
    }

    /// Try and resolve a binding, making a function call if necessary.
    fn handle_identifier(&mut self, ident: &str, expressions: Expressions) -> Binding {
        let binding = self
            .bindings
            .get(ident)
            .expect(&format!("Unknown identifier {}", ident))
            .clone();

        match binding {
            Binding::Expression(expr) => return Binding::Expression(expr.clone()),

            Binding::Function(..) | Binding::NativeFunction(..) => {
                if expressions.len() != 0 {
                    return self.handle_function(&binding, &ident, expressions);
                } else {
                    return binding.clone();
                }
            },
        }
    }

    /// Try and execute a function
    fn handle_function(
        &mut self,
        func: &Binding,
        ident: &str,
        expressions: Expressions
    ) -> Binding
    {
        // TODO: seems lengthy... can this be trimmed down?
        assert!(matches!(
            func,
            Binding::Function(..) | Binding::NativeFunction(..)
        ));

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
                            .map(|expr| match expr {
                                Expr::Argument(num) => args[num].clone(),
                                _ => expr
                            })
                            // Collection needed to avoid recursion limit
                            .collect();
                        self.interpret(contents.into_iter())
                    },

                    Expr::Identifier(_) => self.handle_identifier(ident, Vec::new().into_iter()),
                    Expr::Integer(_) | Expr::Boolean(_) | Expr::Unit => Binding::Expression(body),
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
                            self.interpret(contents.into_iter())
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
}

/// Native variadic function to add numbers
fn add_native(bindings: Bindings) -> Binding {
    let res = bindings.into_iter().fold(0, |acc, next| {
        if let Binding::Expression(Expr::Integer(i)) = next {
            acc + i
        } else {
            panic!("Unexpected argument to add: {:?}", next)
        }
    });

    Binding::Expression(Expr::Integer(res))
}
