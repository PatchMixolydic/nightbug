use std::convert::TryFrom;
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

/// An expression
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
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
            .build_error("unclosed delimiter in file")
            .span_label(location..location + 1, "this delimiter")
            .span_label(eof..eof + 1, "reached end of file before finding match")
            .emit();
    }

    /// Tries to turn a `Token` into an `Expr`
    fn parse_token(&mut self, token: Token) -> Result<Expr, ParseError> {
        let Token { span, kind } = token;
        match kind {
            TokenKind::IdentOrKeyword(id_or_kw) => {
                if let Ok(keyword) = Keyword::try_from(id_or_kw.as_str()) {
                    return Ok(Expr::Keyword(keyword));
                }

                match id_or_kw.as_str() {
                    "true" => Ok(Expr::Boolean(true)),
                    "false" => Ok(Expr::Boolean(false)),
                    _ => Ok(Expr::Identifier(id_or_kw.clone()))
                }
            },

            TokenKind::Integer(i) => Ok(Expr::Integer(i)),

            TokenKind::OpenParen => {
                let mut contents = Vec::new();

                if let Some(next_token) = self.tokens.next() {
                    if next_token.kind == TokenKind::CloseParen {
                        return Ok(Expr::Unit);
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

                let mut prev_token_span_end = span.end - 1;

                loop {
                    let next_token = match self.tokens.next() {
                        Some(next_token) => next_token,

                        None => {
                            self.emit_unclosed_delimiter_err(span.start, prev_token_span_end);
                            return Err(ParseError::UnclosedDelimiter {
                                location: span.start,
                                eof: prev_token_span_end
                            });
                        }
                    };

                    if next_token.kind == TokenKind::CloseParen {
                        break;
                    }

                    prev_token_span_end = next_token.span.end - 1;
                    match self.parse_token(next_token) {
                        Ok(expr) => contents.push(expr),
                        Err(err) => {
                            // probably already emitted an error, propagate it
                            return Err(err);
                        }
                    }
                }

                Ok(Expr::List(contents))
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
