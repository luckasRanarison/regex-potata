use crate::{
    ast::{Node, RangeType},
    error::ParsingError,
};
use std::{
    iter::{self, Peekable},
    str::Chars,
};

pub type Result<T> = std::result::Result<T, ParsingError>;

pub struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Node> {
        self.parse_alternation()
    }

    fn parse_alternation(&mut self) -> Result<Node> {
        let lhs = self.parse_concatenaion()?;

        if let Some('|') = self.peek() {
            let _ = self.next();
            let rhs = self.parse_alternation()?;

            Ok(Node::Alternation(Box::new(lhs), Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    fn parse_concatenaion(&mut self) -> Result<Node> {
        let lhs = self.parse_unary()?;

        match self.peek() {
            Some('|') | Some(')') | None => Ok(lhs),
            Some(_) => {
                let rhs = self.parse_concatenaion()?;
                Ok(Node::Concatenation(Box::new(lhs), Box::new(rhs)))
            }
        }
    }

    fn parse_unary(&mut self) -> Result<Node> {
        let lhs = self.parse_literal()?;

        match self.peek() {
            Some('*') => Ok(self.next_and(Node::Star(Box::new(lhs)))),
            Some('+') => Ok(self.next_and(Node::Plus(Box::new(lhs)))),
            Some('?') => Ok(self.next_and(Node::Optional(Box::new(lhs)))),
            Some('{') => {
                let _ = self.next();

                Ok(Node::Range {
                    inner: Box::new(lhs),
                    range: self.parse_range()?,
                })
            }
            _ => Ok(lhs),
        }
    }

    fn parse_literal(&mut self) -> Result<Node> {
        match self.next() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_class(),
            Some('\\') => self.parse_escape(),
            Some('.') => Ok(Node::Wildcard),
            Some(ch) => Ok(Node::Character(ch)),
            None => Ok(Node::Empty),
        }
    }

    fn parse_group(&mut self) -> Result<Node> {
        let inner = Box::new(self.parse_alternation()?);
        let _ = self.next_expect(')')?;

        Ok(Node::Group(inner))
    }

    fn parse_class(&mut self) -> Result<Node> {
        todo!()
    }

    fn parse_range(&mut self) -> Result<RangeType> {
        if let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_numeric() => self.parse_range_inner(),
                _ => Err(ParsingError::InvalidQuantifier),
            }
        } else {
            Err(ParsingError::UnexpectedEndOfInput)
        }
    }

    fn parse_range_inner(&mut self) -> Result<RangeType> {
        let lower = self.take_number()?;

        match self.next() {
            Some('}') => Ok(self.next_and(RangeType::Exact(lower))),
            Some(',') => match self.peek() {
                Some(ch) if ch.is_numeric() => {
                    let upper = self.take_number()?;
                    let _ = self.next_expect('}')?;

                    Ok(RangeType::Between(lower, upper))
                }
                Some('}') => Ok(self.next_and(RangeType::AtLeast(lower))),
                Some(_) => Err(ParsingError::InvalidQuantifier),
                None => Err(ParsingError::UnexpectedEndOfInput),
            },
            Some(_) => Err(ParsingError::InvalidQuantifier),
            None => Err(ParsingError::UnexpectedEndOfInput),
        }
    }

    fn parse_escape(&mut self) -> Result<Node> {
        match self.next() {
            Some(ch) if needs_escape(ch) => Ok(Node::Character(ch)),
            Some(_) => Err(ParsingError::InvalidEscapeSequence),
            None => Err(ParsingError::UnexpectedEndOfInput),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn next_and<T>(&mut self, value: T) -> T {
        self.chars.next();
        value
    }

    fn next_expect(&mut self, ch: char) -> Result<char> {
        self.chars
            .next()
            .filter(|&next| next == ch)
            .ok_or(ParsingError::MissingCharacter(ch))
    }

    fn take_number(&mut self) -> Result<usize> {
        iter::from_fn(|| self.chars.next_if(|ch| ch.is_numeric()))
            .collect::<String>()
            .parse::<usize>()
            .or(Err(ParsingError::InvalidQuantifier))
    }
}

fn needs_escape(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '[' | ']' | '(' | ')' | '{' | '}' | '.' | '?' | '+' | '*'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chars() {
        let ast = Parser::new("ok!").parse().unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('o')),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('k')),
                Box::new(Node::Character('!')),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_unary() {
        let ast = Parser::new("les?").parse().unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('l')),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('e')),
                Box::new(Node::Optional(Box::new(Node::Character('s')))),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_alternation() {
        let ast = Parser::new("la|le").parse().unwrap();
        let expected = Node::Alternation(
            Box::new(Node::Concatenation(
                Box::new(Node::Character('l')),
                Box::new(Node::Character('a')),
            )),
            Box::new(Node::Concatenation(
                Box::new(Node::Character('l')),
                Box::new(Node::Character('e')),
            )),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_group_alternation() {
        let ast = Parser::new("l(a|e)").parse().unwrap();
        let expected = Node::Concatenation(
            Box::new(Node::Character('l')),
            Box::new(Node::Group(Box::new(Node::Alternation(
                Box::new(Node::Character('a')),
                Box::new(Node::Character('e')),
            )))),
        );

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_quantifier() {
        let ast = Parser::new("1{2,5}").parse().unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: RangeType::Between(2, 5),
        };

        assert_eq!(ast, expected);

        let ast = Parser::new("1{5}").parse().unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: RangeType::Exact(5),
        };

        assert_eq!(ast, expected);

        let ast = Parser::new("1{5,}").parse().unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: RangeType::AtLeast(5),
        };

        assert_eq!(ast, expected);
    }
}
