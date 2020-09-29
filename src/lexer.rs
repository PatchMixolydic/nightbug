use std::iter::Peekable;
use std::str::Chars;

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

impl Token {
    /// Try and yield the token that best fits the input
    fn lex(source: &mut Peekable<Chars>) -> Option<Self> {
        match source.next()? {
            c @ ('A'..='Z' | 'a'..='z' | '_') => {
                // TODO: be more permissive w/ identifiers
                let ident_or_keyword = {
                    let res = String::from(c);
                    consume_ident(source, res)
                };
                Some(Self::IdentOrKeyword(ident_or_keyword))
            },

            c @ '0'..='9' => {
                // TODO: negative integers
                let res = String::from(c);
                Some(Self::Integer(consume_integer(source, res)))
            },

            '(' => Some(Self::OpenParen),
            ')' => Some(Self::CloseParen),
            ' ' | '\t' | '\n' | '\r' => Some(Self::Whitespace),
            c => todo!("{}", c)
        }
    }
}

/// Turn a source stream into a `Vec` of `Token`s
pub fn lex(source: &str) -> Vec<Token> {
    let mut source_chars = source.chars().peekable();
    let mut res = Vec::new();
    while let Some(token) = Token::lex(&mut source_chars) {
        if token != Token::Whitespace {
            res.push(token);
        }
    }

    res
}

fn consume_ident(source: &mut Peekable<Chars>, mut res: String) -> String {
    // TODO: be more permissive
    while let Some('A'..='Z' | 'a'..='z' | '_' | '0'..='9') = source.peek() {
        res.push(source.next().unwrap());
    }
    res
}

fn consume_integer(source: &mut Peekable<Chars>, mut res: String) -> i32 {
    while let Some('0'..='9') = source.peek() {
        res.push(source.next().unwrap());
    }
    res.parse::<i32>()
        .expect(&format!("Failed to parse i32: {}", res))
}
