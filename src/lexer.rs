use std::{
    iter::{Enumerate, Peekable},
    num::ParseIntError,
    ops::Range,
    str::Chars
};
use thiserror::Error;

use crate::errors::DiagnosticsContext;

type CharStream<'a> = Peekable<Enumerate<Chars<'a>>>;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character {0}")]
    UnexpectedChar(char, usize),
    #[error("Failed to parse {0}")]
    CouldntParseInt(String, #[source] ParseIntError)
}

/// A lexical token read from a source stream
#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
    /// Identifier or keyword ("foo", "define", "true")
    IdentOrKeyword(String),
    /// Integer ("4", "535325", "0")
    Integer(i32),
    /// Open parenthesis ("(")
    OpenParen,
    /// Close parenthesis (")")
    CloseParen,
    /// Internally used for whitespace (" ")
    Whitespace
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub span: Range<usize>,
    pub kind: TokenKind
}

impl Token {
    fn new(span: Range<usize>, kind: TokenKind) -> Self {
        Self { span, kind }
    }
    /// Try and yield the token that best fits the input
    fn lex(
        source: &mut CharStream,
        error_ctx: &DiagnosticsContext
    ) -> Result<Option<Self>, LexError>
    {
        // For convenience/reducing parens
        macro_rules! ok_some_self {
            ($span:expr, $kind:expr) => {
                Ok(Some(Self::new($span, $kind)))
            };
        }

        let (idx, c) = match source.peek() {
            // Tuple is deconstructed here to copy the fields
            Some(x) => (x.0, x.1),
            None => return Ok(None)
        };
        // Span for c, since it's the most common
        let span_c = idx..idx + 1;

        match c {
            'A'..='Z' | 'a'..='z' | '_' => {
                // TODO: be more permissive w/ identifiers
                Ok(Some(consume_ident(source)))
            },

            '0'..='9' => {
                // TODO: negative integers
                match consume_integer(source, error_ctx) {
                    Ok(res) => Ok(Some(res)),
                    Err((num_str, err)) => Err(LexError::CouldntParseInt(num_str, err))
                }
            },

            '(' => {
                source.next();
                ok_some_self!(span_c, TokenKind::OpenParen)
            },
            ')' => {
                source.next();
                ok_some_self!(span_c, TokenKind::CloseParen)
            },
            ' ' | '\t' | '\n' | '\r' => {
                source.next();
                ok_some_self!(span_c, TokenKind::Whitespace)
            },
            _ => {
                error_ctx
                    .build_error_span(span_c, "unexpected character")
                    .emit();
                Err(LexError::UnexpectedChar(c, idx))
            }
        }
    }
}

/// Turn a source stream into a `Vec` of `Token`s
pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut source_chars = source.chars().enumerate().peekable();
    let error_ctx = DiagnosticsContext::new(source, None);
    let mut res = Vec::new();

    while let Some(token) = Token::lex(&mut source_chars, &error_ctx)? {
        if token.kind != TokenKind::Whitespace {
            res.push(token);
        }
    }

    Ok(res)
}

fn consume_ident(source: &mut CharStream) -> Token {
    let mut res = String::new();
    let start = source.peek().unwrap().0;

    while let Some((_, c)) = source.peek() {
        // This if statement is seperated from the while statement
        // for readability purposes
        // TODO: be more permissive
        if matches!(c, 'A'..='Z' | 'a'..='z' | '_' | '0'..='9') {
            // nb. we are using source.peek() above
            res.push(source.next().unwrap().1);
        } else {
            break;
        }
    }

    Token::new(start..start + res.len(), TokenKind::IdentOrKeyword(res))
}

fn consume_integer(
    source: &mut CharStream,
    error_ctx: &DiagnosticsContext
) -> Result<Token, (String, ParseIntError)>
{
    let start = source.peek().unwrap().0;
    let mut num_str = String::new();

    while let Some((_, '0'..='9')) = source.peek() {
        num_str.push(source.next().unwrap().1);
    }

    num_str
        .parse::<i32>()
        .map(|res| Token::new(start..start + num_str.len(), TokenKind::Integer(res)))
        .or_else(|err| {
            error_ctx
                .build_ice_span(
                    start..start + num_str.len(),
                    &format!("could not parse {} into an integer", num_str)
                )
                .note(&format!("str::parse::<i32> says: {}", err))
                .emit();
            Err((num_str, err))
        })
}
