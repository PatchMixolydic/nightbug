use std::convert::TryFrom;

use crate::lexer::Token;

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

struct Parser {
    // Is there a better way?
    tokens: Box<dyn Iterator<Item = Token>>
}

impl Parser {
    /// Tries to turn a `Token` into an `Expr`
    fn parse_token(&mut self, token: Token) -> Option<Expr> {
        match token {
            Token::IdentOrKeyword(id_or_kw) => {
                if let Ok(keyword) = Keyword::try_from(id_or_kw.as_str()) {
                    return Some(Expr::Keyword(keyword));
                }

                match id_or_kw.as_str() {
                    "true" => Some(Expr::Boolean(true)),
                    "false" => Some(Expr::Boolean(false)),
                    _ => Some(Expr::Identifier(id_or_kw.clone()))
                }
            },

            Token::Integer(i) => Some(Expr::Integer(i)),

            Token::OpenParen => {
                let mut contents = Vec::new();
                let next_token = self.tokens.next();

                if let Some(Token::CloseParen) = next_token {
                    return Some(Expr::Unit);
                } else if let Some(token) = next_token {
                    contents.push(self.parse_token(token).expect("Unexpected EOF"));
                } else {
                    panic!("unexpected EOF");
                }

                while let Some(token) = self.tokens.next() {
                    if token == Token::CloseParen {
                        break;
                    }

                    contents.push(self.parse_token(token).expect("Unexpected EOF"));
                }

                Some(Expr::List(contents))
            },

            Token::CloseParen => unimplemented!(),
            Token::Whitespace => unreachable!()
        }
    }

    /// Convenience for parsing the next token in self.tokens
    fn parse_next(&mut self) -> Option<Expr> {
        let next_token = self.tokens.next();
        if next_token.is_none() {
            return None;
        }
        self.parse_token(next_token.unwrap())
    }
}

/// Parse the given `Vec` of `Token`s into a `Vec` of `Expr`s.
pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
    let mut res = Vec::new();
    let mut parser = Parser {
        tokens: Box::new(tokens.into_iter())
    };

    while let Some(token) = parser.parse_next() {
        res.push(token);
    }

    res
}
