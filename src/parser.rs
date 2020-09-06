use std::convert::TryFrom;

use crate::lexer::Token;

#[derive(Debug)]
pub enum Keyword {
    True,
    False
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Keyword(Keyword),
    Identifier(String),
    Integer(i32),
    Unit,
    List(Vec<Expr>)
}

struct Parser {
    tokens: Box<dyn Iterator<Item = Token>>
}

impl Parser {
    fn parse_token(&mut self, token: Token) -> Option<Expr> {
        match token {
            Token::IdentOrKeyword(id_or_kw) => {
                match Keyword::try_from(id_or_kw.as_str()) {
                    Ok(keyword) => Some(Expr::Keyword(keyword)),
                    Err(_) => Some(Expr::Identifier(id_or_kw.clone())),
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

    fn parse_next(&mut self) -> Option<Expr> {
        let next_token = self.tokens.next();
        if next_token.is_none() {
            return None;
        }
        self.parse_token(next_token.unwrap())
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
    let mut res = Vec::new();
    let mut parser = Parser { tokens: Box::new(tokens.into_iter()) };

    while let Some(token) = parser.parse_next() {
        res.push(token);
    }

    res
}
