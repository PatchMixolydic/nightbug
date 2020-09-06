use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    IdentOrKeyword(String),
    Integer(i32),
    OpenParen,
    CloseParen,
    Whitespace
}

impl Token {
    fn lex(source: &mut Peekable<Chars>) -> Option<Self> {
        match source.next()? {
            c @ ('A'..='Z' | 'a'..='z' | '_') => {
                let ident_or_keyword = {
                    let res = String::from(c);
                    consume_ident(source, res)
                };
                Some(Self::IdentOrKeyword(ident_or_keyword))
            },

            c @ '0'..='9' => {
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
    while let Some('A'..='Z' | 'a'..='z' | '_' | '0'..='9') = source.peek() {
        res.push(source.next().unwrap());
    }
    res
}

fn consume_integer(source: &mut Peekable<Chars>, mut res: String) -> i32 {
    while let Some('0'..='9') = source.peek() {
        res.push(source.next().unwrap());
    }
    res.parse::<i32>().expect(&format!("Failed to parse i32: {}", res))
}
