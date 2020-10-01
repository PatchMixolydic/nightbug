use std::{
    iter::{Enumerate, Peekable},
    num::ParseIntError,
    ops::Range,
    str::Chars
};
use thiserror::Error;

use crate::errors::DiagnosticsContext;

type CharStream<'a> = Peekable<Enumerate<Chars<'a>>>;

/// Signals error encountered during lexing.
#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character {0}")]
    UnexpectedChar(char, usize),
    #[error("Failed to parse {0}")]
    CouldntParseInt(String, #[source] ParseIntError)
}

/// Distinguishes between `Token`s.
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

/// A lexical token read from a source stream.
#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    /// Indicates the region in the source code
    /// which this token corresponds to.
    pub span: Range<usize>,
    pub kind: TokenKind
}

impl Token {
    fn new(span: Range<usize>, kind: TokenKind) -> Self {
        Self { span, kind }
    }
}

/// Keeps the lexer state during lexing.
struct Lexer<'src> {
    chars: CharStream<'src>,
    error_ctx: DiagnosticsContext<'src>
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            chars: source.chars().enumerate().peekable(),
            error_ctx: DiagnosticsContext::new(source, None)
        }
    }

    /// Try and yield the token that best fits the input.
    /// Returns Ok(Some(...)) if a token was lexed,
    /// Ok(None) if the source stream was exhausted,
    /// or Err(...) if an error occurred.
    fn lex_one_token(&mut self) -> Result<Option<Token>, LexError> {
        // For convenience/reducing parens
        macro_rules! ok_some_token {
            ($span:expr, $kind:expr) => {
                Ok(Some(Token::new($span, $kind)))
            };
        }

        let (idx, c) = match self.chars.peek() {
            // Tuple is deconstructed here to copy the fields
            Some(x) => (x.0, x.1),
            None => return Ok(None)
        };
        // Span for c, since it's the most common
        let span_c = idx..idx + 1;

        match c {
            'A'..='Z' | 'a'..='z' | '_' => {
                // TODO: be more permissive w/ identifiers
                Ok(Some(self.consume_ident()))
            },

            '0'..='9' => {
                // TODO: negative integers
                match self.consume_integer() {
                    Ok(res) => Ok(Some(res)),
                    Err((num_str, err)) => Err(LexError::CouldntParseInt(num_str, err))
                }
            },

            '(' => {
                self.chars.next();
                ok_some_token!(span_c, TokenKind::OpenParen)
            },
            ')' => {
                self.chars.next();
                ok_some_token!(span_c, TokenKind::CloseParen)
            },
            ' ' | '\t' | '\n' | '\r' => {
                self.chars.next();
                ok_some_token!(span_c, TokenKind::Whitespace)
            },
            _ => {
                self.error_ctx
                    .build_error_span(span_c, "unexpected character")
                    .emit();
                Err(LexError::UnexpectedChar(c, idx))
            }
        }
    }

    /// Take every character that could be considered part of an identifier
    /// and produce an `IdentOrKeyword` token.
    fn consume_ident(&mut self) -> Token {
        let mut res = String::new();
        let start = self.chars.peek().unwrap().0;

        while let Some((_, c)) = self.chars.peek() {
            // This if statement is seperated from the while statement
            // for readability purposes
            // TODO: be more permissive
            if matches!(c, 'A'..='Z' | 'a'..='z' | '_' | '0'..='9') {
                // nb. we are using source.peek() above
                res.push(self.chars.next().unwrap().1);
            } else {
                break;
            }
        }

        Token::new(start..start + res.len(), TokenKind::IdentOrKeyword(res))
    }

    /// Take every character that could be considered part of an integer
    /// and produce an `Integer` token.
    /// On failure, returns a tuple including the number that failed to parse
    /// and the error produced by the parse function.
    fn consume_integer(&mut self) -> Result<Token, (String, ParseIntError)> {
        let start = self.chars.peek().unwrap().0;
        let mut num_str = String::new();

        while let Some((_, '0'..='9')) = self.chars.peek() {
            num_str.push(self.chars.next().unwrap().1);
        }

        num_str
            .parse::<i32>()
            .map(|res| Token::new(start..start + num_str.len(), TokenKind::Integer(res)))
            .or_else(|err| {
                self.error_ctx
                    .build_ice_span(
                        start..start + num_str.len(),
                        &format!("could not parse {} into an integer", num_str)
                    )
                    .note(&format!("str::parse::<i32> says: {}", err))
                    .emit();
                Err((num_str, err))
            })
    }
}

/// Turn a source stream into a `Vec` of `Token`s
pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    let mut res = Vec::new();

    while let Some(token) = lexer.lex_one_token()? {
        if token.kind != TokenKind::Whitespace {
            res.push(token);
        }
    }

    Ok(res)
}
