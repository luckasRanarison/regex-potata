use crate::{
    ast::{CharacterClass, ClassType, Node, Range},
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
            Some('*') => self.next_and(Node::Star(Box::new(lhs))),
            Some('+') => self.next_and(Node::Plus(Box::new(lhs))),
            Some('?') => self.next_and(Node::Optional(Box::new(lhs))),
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
            Some('.') => Ok(Node::Wildcard),
            Some('\\') => Ok(Node::Character(self.parse_escape()?)),
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
        if self.peek().is_some() {
            let negate = self.next_if('^');
            let mut inner = Vec::new();

            while self.peek() != Some(']') {
                inner.push(self.parse_class_inner()?);
            }

            let _ = self.next_expect(']');

            Ok(Node::CharacterClass(CharacterClass { negate, inner }))
        } else {
            Err(ParsingError::UnexpectedEndOfInput)
        }
    }

    fn parse_class_inner(&mut self) -> Result<ClassType> {
        let first = self.parse_character()?;

        if self.next_if('-') {
            let second = self.parse_character()?;

            if first > second {
                Err(ParsingError::RangeOutOfOrder)
            } else {
                Ok(ClassType::Range(first, second))
            }
        } else {
            Ok(ClassType::Atom(first))
        }
    }

    fn parse_character(&mut self) -> Result<char> {
        match self.next() {
            Some('\\') => self.parse_escape(),
            Some(ch) => Ok(ch),
            None => Err(ParsingError::UnexpectedEndOfInput),
        }
    }

    fn parse_range(&mut self) -> Result<Range> {
        if let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_numeric() => self.parse_range_inner(),
                _ => Err(ParsingError::InvalidQuantifier),
            }
        } else {
            Err(ParsingError::UnexpectedEndOfInput)
        }
    }

    fn parse_range_inner(&mut self) -> Result<Range> {
        let lower = self.take_number()?;

        match self.next() {
            Some('}') => self.next_and(Range::new(lower, Some(lower))),
            Some(',') => match self.peek() {
                Some(ch) if ch.is_numeric() => {
                    let upper = self.take_number()?;
                    let _ = self.next_expect('}')?;

                    Ok(Range::new(lower, Some(upper)))
                }
                Some('}') => self.next_and(Range::new(lower, None)),
                Some(_) => Err(ParsingError::InvalidQuantifier),
                None => Err(ParsingError::UnexpectedEndOfInput),
            },
            Some(_) => Err(ParsingError::InvalidQuantifier),
            None => Err(ParsingError::UnexpectedEndOfInput),
        }
    }

    fn parse_escape(&mut self) -> Result<char> {
        match self.next() {
            Some(ch) if needs_escape(ch) => Ok(ch),
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

    fn next_if(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.chars.next();
            true
        } else {
            false
        }
    }

    fn next_and<T>(&mut self, value: T) -> Result<T> {
        self.chars.next();
        Ok(value)
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
    use crate::ast::CharacterClass;

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
            range: Range::new(2, Some(5)),
        };

        assert_eq!(ast, expected);

        let ast = Parser::new("1{5}").parse().unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: Range::new(5, Some(5)),
        };

        assert_eq!(ast, expected);

        let ast = Parser::new("1{5,}").parse().unwrap();
        let expected = Node::Range {
            inner: Box::new(Node::Character('1')),
            range: Range::new(5, None),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_character_class() {
        let ast = Parser::new(r#"[bar\\]"#).parse().unwrap();
        let expected = Node::CharacterClass(CharacterClass {
            negate: false,
            inner: vec![
                ClassType::Atom('b'),
                ClassType::Atom('a'),
                ClassType::Atom('r'),
                ClassType::Atom('\\'),
            ],
        });

        assert_eq!(ast, expected);

        let ast = Parser::new(r#"[^a-zA-Z.]"#).parse().unwrap();
        let expected = Node::CharacterClass(CharacterClass {
            negate: true,
            inner: vec![
                ClassType::Range('a', 'z'),
                ClassType::Range('A', 'Z'),
                ClassType::Atom('.'),
            ],
        });

        assert_eq!(ast, expected);
    }
}
