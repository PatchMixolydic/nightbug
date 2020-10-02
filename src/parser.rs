use std::{convert::TryFrom, ops::Range};
use thiserror::Error;

use crate::{
    errors::DiagnosticsContext,
    lexer::{Token, TokenKind}
};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unclosed delimiter at character {location}")]
    UnclosedDelimiter { location: usize, eof: usize },
    #[error("Unexpected closing delimiter at character {0}")]
    UnexpectedCloseDelimiter(usize)
}

/// A keyword
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    /// Create a binding
    Define,
    /// Declare a function
    Fn
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "define" => Ok(Self::Define),
            "fn" => Ok(Self::Fn),
            _ => Err(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprKind {
    /// eg. "define"
    Keyword(Keyword),
    /// eg. "foo"
    Identifier(String),
    /// eg. "456"
    Integer(i32),
    /// eg. "false"
    Boolean(bool),
    /// ()
    Unit,
    /// Used internally for functions
    Argument(usize),
    /// S-expression (eg. "(add 2 2)")
    List(Vec<Expr>)
}

/// An expression
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr {
    pub span: Range<usize>,
    pub kind: ExprKind
}

impl Expr {
    pub fn new(span: Range<usize>, kind: ExprKind) -> Self {
        Self { span, kind }
    }

    /// Convenience function to create a keyword expression
    pub fn keyword(span: Range<usize>, keyword: Keyword) -> Self {
        Self::new(span, ExprKind::Keyword(keyword))
    }

    /// Convenience function to create an identifier expression
    pub fn identifier(span: Range<usize>, ident: String) -> Self {
        Self::new(span, ExprKind::Identifier(ident))
    }

    /// Convenience function to create an integer expression
    pub fn integer(span: Range<usize>, num: i32) -> Self {
        Self::new(span, ExprKind::Integer(num))
    }

    /// Convenience function to create a boolean expression
    pub fn boolean(span: Range<usize>, b: bool) -> Self {
        Self::new(span, ExprKind::Boolean(b))
    }

    /// Convenience function to create a unit value expression
    pub fn unit(span: Range<usize>) -> Self {
        Self::new(span, ExprKind::Unit)
    }

    /// Convenience function to create an argument expression
    pub fn argument(span: Range<usize>, idx: usize) -> Self {
        Self::new(span, ExprKind::Argument(idx))
    }

    /// Convenience function to create a list expression
    pub fn list(span: Range<usize>, exprs: Vec<Expr>) -> Self {
        Self::new(span, ExprKind::List(exprs))
    }
}

struct Parser<'src> {
    // Is there a better way?
    tokens: Box<dyn Iterator<Item = Token>>,
    error_ctx: DiagnosticsContext<'src>
}

impl<'src> Parser<'src> {
    fn new(tokens: Vec<Token>, code: &'src str) -> Self {
        Self {
            tokens: Box::new(tokens.into_iter()),
            error_ctx: DiagnosticsContext::new(code, None)
        }
    }

    fn emit_unclosed_delimiter_err(&self, location: usize, eof: usize) {
        self.error_ctx
            .build_error("unclosed delimiter detected")
            .span_label(location..location + 1, "this delimiter")
            .span_label(eof..eof + 1, "reached end of file before finding a match")
            .emit();
    }

    /// Tries to turn a `Token` into an `Expr`
    fn parse_token(&mut self, token: Token) -> Result<Expr, ParseError> {
        let Token { span, kind } = token;
        match kind {
            TokenKind::IdentOrKeyword(id_or_kw) => {
                if let Ok(keyword) = Keyword::try_from(id_or_kw.as_str()) {
                    return Ok(Expr::keyword(span, keyword));
                }

                match id_or_kw.as_str() {
                    "true" => Ok(Expr::boolean(span, true)),
                    "false" => Ok(Expr::boolean(span, false)),
                    _ => Ok(Expr::identifier(span, id_or_kw.clone()))
                }
            },

            TokenKind::Integer(i) => Ok(Expr::integer(span, i)),

            TokenKind::OpenParen => {
                let mut contents = Vec::new();

                if let Some(next_token) = self.tokens.next() {
                    if next_token.kind == TokenKind::CloseParen {
                        return Ok(Expr::new(span.start..span.end + 1, ExprKind::Unit));
                    } else {
                        contents.push(self.parse_token(next_token)?);
                    }
                } else {
                    // Span end is one after our token
                    self.emit_unclosed_delimiter_err(span.start, span.end - 1);
                    return Err(ParseError::UnclosedDelimiter {
                        location: span.start,
                        eof: span.end - 1
                    });
                }

                // nb. the unwrap here should be infallible --
                // all branches in the above if expression either return or push
                // to contents
                let mut prev_expr_span_end = contents.last().unwrap().span.end - 1;

                loop {
                    let next_token = match self.tokens.next() {
                        Some(next_token) => next_token,

                        None => {
                            self.emit_unclosed_delimiter_err(span.start, prev_expr_span_end);
                            return Err(ParseError::UnclosedDelimiter {
                                location: span.start,
                                eof: prev_expr_span_end
                            });
                        }
                    };

                    if next_token.kind == TokenKind::CloseParen {
                        break;
                    }

                    match self.parse_token(next_token) {
                        Ok(expr) => {
                            prev_expr_span_end = expr.span.end;
                            contents.push(expr);
                        },

                        Err(err) => {
                            // probably already emitted an error, propagate it
                            return Err(err);
                        }
                    }
                }

                Ok(Expr::new(
                    span.start..prev_expr_span_end,
                    ExprKind::List(contents)
                ))
            },

            TokenKind::CloseParen => {
                self.error_ctx
                    .build_error_span(0..0, "unexpected closing parenthesis")
                    .emit();
                Err(ParseError::UnexpectedCloseDelimiter(0))
            },

            TokenKind::Whitespace => unreachable!()
        }
    }

    /// Convenience for parsing the next token in self.tokens
    fn parse_next(&mut self) -> Result<Option<Expr>, ParseError> {
        let next_token = self.tokens.next();
        match next_token {
            Some(token) => Ok(Some(self.parse_token(token)?)),
            None => Ok(None)
        }
    }
}

/// Parse the given `Vec` of `Token`s into a `Vec` of `Expr`s.
pub fn parse(tokens: Vec<Token>, code: &str) -> Result<Vec<Expr>, ParseError> {
    let mut res = Vec::new();
    let mut parser = Parser::new(tokens, code);

    while let Some(token) = parser.parse_next()? {
        res.push(token);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn fail_unclosed() {
        let code = "(add 2 (second 3 4)) (foobar 1";
        let res = parse(lex(code).unwrap(), code);
        assert!(matches!(res, Err(ParseError::UnclosedDelimiter { .. })));
    }
}
