use std::{
    iter::{Enumerate, Peekable},
    num::ParseIntError,
    str::Chars
};
use thiserror::Error;

use crate::errors::DiagnosticsContext;

type CharStream<'a> = Peekable<Enumerate<Chars<'a>>>;

/// A lexical token read from a source stream
#[derive(Debug, Eq, PartialEq)]
pub enum Token {
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

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character {0}")]
    UnexpectedChar(char, usize),
    #[error("Failed to parse {0}")]
    CouldntParseInt(String, #[source] ParseIntError)
}

impl Token {
    /// Try and yield the token that best fits the input
    fn lex(
        source: &mut CharStream,
        error_ctx: &DiagnosticsContext
    ) -> Result<Option<Self>, LexError>
    {
        let (idx, c) = match source.next() {
            Some(x) => x,
            None => return Ok(None)
        };

        match c {
            'A'..='Z' | 'a'..='z' | '_' => {
                // TODO: be more permissive w/ identifiers
                let ident_or_keyword = {
                    let res = String::from(c);
                    consume_ident(source, res)
                };
                Ok(Some(Self::IdentOrKeyword(ident_or_keyword)))
            },

            '0'..='9' => {
                // TODO: negative integers
                let res = String::from(c);
                match consume_integer(source, res, error_ctx) {
                    Ok(res) => Ok(Some(Self::Integer(res))),
                    Err((num, err)) => Err(LexError::CouldntParseInt(num, err))
                }
            },

            '(' => Ok(Some(Self::OpenParen)),
            ')' => Ok(Some(Self::CloseParen)),
            ' ' | '\t' | '\n' | '\r' => Ok(Some(Self::Whitespace)),
            _ => {
                error_ctx
                    .build_error_span(idx..idx + 1, "unexpected character")
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
        if token != Token::Whitespace {
            res.push(token);
        }
    }

    Ok(res)
}

fn consume_ident(source: &mut CharStream, mut res: String) -> String {
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
    res
}

fn consume_integer(
    source: &mut CharStream,
    mut res: String,
    error_ctx: &DiagnosticsContext
) -> Result<i32, (String, ParseIntError)>
{
    let start = source.peek().unwrap().0 - 1;
    let mut end = start;

    while let Some((idx, '0'..='9')) = source.peek() {
        end = *idx;
        res.push(source.next().unwrap().1);
    }

    res.parse::<i32>().or_else(|err| {
        error_ctx
            .build_ice_span(
                start..end + 1,
                &format!("could not parse {} into an integer", res)
            )
            .note(&format!("str::parse::<i32> says: {}", err))
            .emit();
        Err((res, err))
    })
}
