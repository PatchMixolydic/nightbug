#[cfg(test)]
mod tests;

use std::{collections::HashMap, ops::Range};
use thiserror::Error;

use crate::{
    errors::DiagnosticsContext,
    parser::{Expr, ExprKind}
};

type Expressions = std::vec::IntoIter<Expr>;
type Bindings = std::vec::IntoIter<Binding>;
type InterpResult = Result<Binding, InterpreterError>;

/// The value of a binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Binding {
    /// An expression (ex. "(add 2 2)")
    Expression(Expr),
    /// A function defined in Nightbug
    Function(usize /* num_arguments */, Expr),
    /// A function defined in Rust
    NativeFunction(Option<usize>, fn(Bindings) -> InterpResult)
}

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Unknown identifier {0}")]
    UnknownIdentifier(String),
    #[error("Wrong number of arguments for {ident} (expected {expected}, got {got})")]
    WrongNumArgs {
        ident: String,
        expected: usize,
        got: usize
    },
    #[error("Invalid argument provided to function {0}: {1:?}")]
    InvalidArgument(String, Binding)
}

pub struct Interpreter<'src> {
    bindings: HashMap<String, Binding>,
    error_ctx: DiagnosticsContext<'src>
}

impl<'src> Interpreter<'src> {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("add".to_string(), Binding::NativeFunction(None, add_native));
        bindings.insert(
            "second".to_string(),
            Binding::Function(2, Expr::argument(0..0, 1))
        );

        Self {
            bindings,
            error_ctx: DiagnosticsContext::new("", None)
        }
    }

    /// Interpret a given list of expressions.
    /// Takes in source code for debugging.
    pub fn interpret_with_source(
        &mut self,
        expressions: Vec<Expr>,
        source: &'src str
    ) -> InterpResult {
        self.error_ctx.set_src(source);
        self.interpret(expressions.into_iter())
    }

    /// Interpret a given iterator over expressions.
    /// Returns a binding as the result of the evaluation.
    fn interpret(&mut self, mut expressions: Expressions) -> InterpResult {
        let Expr { span, kind } = match expressions.next() {
            Some(expr) => expr,
            None => return Ok(Binding::Expression(Expr::unit(0..0)))
        };

        match kind {
            ExprKind::Integer(_) | ExprKind::Boolean(_) | ExprKind::Unit => {
                Ok(Binding::Expression(Expr::new(span, kind)))
            },

            ExprKind::List(inner_expressions) => self.interpret(inner_expressions.into_iter()),
            ExprKind::Identifier(ident) => self.handle_identifier(&ident, span, expressions),
            ExprKind::Keyword(_) => todo!(),
            ExprKind::Argument(_) => unreachable!()
        }
    }

    /// Try and resolve a binding, making a function call if necessary.
    fn handle_identifier(
        &mut self,
        ident: &str,
        span: Range<usize>,
        expressions: Expressions
    ) -> InterpResult {
        let binding = match self.bindings.get(ident) {
            Some(res) => res.clone(),
            None => {
                self.error_ctx
                    .build_error(&format!("unknown identifier `{}`", ident))
                    .span_label(span.clone(), "not found in this scope")
                    .emit();

                return Err(InterpreterError::UnknownIdentifier(ident.to_string()));
            }
        };

        match binding {
            Binding::Expression(expr) => return Ok(Binding::Expression(expr.clone())),

            Binding::Function(..) | Binding::NativeFunction(..) => {
                if expressions.len() != 0 {
                    return self.handle_function(&binding, &ident, span, expressions);
                } else {
                    return Ok(binding.clone());
                }
            },
        }
    }

    /// Try and execute a function
    fn handle_function(
        &mut self,
        func: &Binding,
        ident: &str,
        name_span: Range<usize>,
        expressions: Expressions
    ) -> InterpResult {
        // TODO: seems lengthy... can this be trimmed down?
        assert!(matches!(
            func,
            Binding::Function(..) | Binding::NativeFunction(..)
        ));

        match func {
            Binding::Function(num_arguments, body) => {
                if expressions.len() != *num_arguments {
                    let num_args_given = expressions.len();
                    let mut msg = self.error_ctx.build_error(&format!(
                        "wrong number of arguments for function (expected {}, got {})",
                        num_arguments, num_args_given
                    ));

                    if num_args_given != 0 {
                        let args: Vec<Expr> = expressions.collect();
                        // Construct a span across the argument list
                        // These unwraps should be safe since the list is nonempty
                        let span = args.first().unwrap().span.start..args.last().unwrap().span.end;
                        msg = msg.span_label(span, &format!("got {} arguments", num_args_given));
                    }

                    msg.span_label(name_span, &format!("expected {} arguments", num_arguments))
                        .emit();

                    return Err(InterpreterError::WrongNumArgs {
                        ident: ident.to_string(),
                        expected: *num_arguments,
                        got: num_args_given
                    });
                }

                let mut body = body.clone();
                let args: Vec<Expr> = expressions.collect();

                if let ExprKind::Argument(num) = body.kind {
                    body = args[num].clone();
                }

                match body.kind {
                    ExprKind::List(contents) => {
                        let contents: Vec<Expr> = contents
                            .into_iter()
                            .map(|expr| match expr.kind {
                                ExprKind::Argument(num) => args[num].clone(),
                                _ => expr
                            })
                            // Collection needed to avoid recursion limit
                            .collect();
                        self.interpret(contents.into_iter())
                    },

                    ExprKind::Identifier(_) => {
                        self.handle_identifier(ident, body.span, Vec::new().into_iter())
                    },

                    ExprKind::Integer(_) | ExprKind::Boolean(_) | ExprKind::Unit => {
                        Ok(Binding::Expression(body))
                    },

                    _ => todo!()
                }
            },

            Binding::NativeFunction(maybe_num_arguments, func) => {
                if let Some(num_arguments) = maybe_num_arguments {
                    if expressions.len() != *num_arguments {
                        self.error_ctx
                            .build_error(&format!(
                                "wrong number of arguments for function (expected {}, got {})",
                                num_arguments,
                                expressions.len()
                            ))
                            .span_label(name_span, &format!("expected {} arguments", num_arguments))
                            .note(&format!(
                                "cannot show definition for {} because it is a built-in function",
                                ident
                            ))
                            .emit();

                        return Err(InterpreterError::WrongNumArgs {
                            ident: ident.to_string(),
                            expected: *num_arguments,
                            got: expressions.len()
                        });
                    }
                }

                let mut bindings = Vec::with_capacity(expressions.len());

                for expr in expressions {
                    let binding = match expr.kind {
                        ExprKind::List(contents) => self.interpret(contents.into_iter())?,
                        _ => Binding::Expression(expr)
                    };
                    bindings.push(binding);
                }

                func(bindings.into_iter())
            },

            _ => unreachable!()
        }
    }
}

/// Native variadic function to add numbers
fn add_native(bindings: Bindings) -> InterpResult {
    // TODO: HACK: get this from interpreter somehow!
    let error_ctx = DiagnosticsContext::new("", None);
    let mut res = 0;

    for binding in bindings {
        match binding {
            Binding::Expression(Expr {
                kind: ExprKind::Integer(i),
                ..
            }) => {
                res += i;
            },

            _ => {
                error_ctx
                    .build_error(&format!("unexpected argument to `add`:{:?}", binding))
                    .note("cannot highlight the argument in your code right now, my apologies")
                    .note("`add` only expects integers as arguments")
                    .emit();
                return Err(InterpreterError::InvalidArgument(
                    "add".to_string(),
                    binding
                ));
            }
        }
    }

    Ok(Binding::Expression(Expr::integer(0..0, res)))
}
