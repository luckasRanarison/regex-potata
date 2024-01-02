use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Node {
    Empty,
    Alternation(Box<Node>, Box<Node>),
    Concatenation(Box<Node>, Box<Node>),
    Star(Box<Node>),
    Plus(Box<Node>),
    Optional(Box<Node>),
    Range { inner: Box<Node>, range: Range },
    Group(Box<Node>),
    Wildcard,
    Character(char),
    CharacterClass(CharacterClass),
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub min: usize,
    pub max: Option<usize>,
}

impl Range {
    pub fn new(min: usize, max: Option<usize>) -> Self {
        Self { min, max }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassType {
    Atom(char),
    Range(char, char),
}

impl fmt::Display for ClassType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassType::Atom(ch) => write!(f, "{ch}"),
            ClassType::Range(lower, upper) => write!(f, "{lower}-{upper}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterClass {
    pub negate: bool,
    pub inner: Vec<ClassType>,
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}{}]",
            if self.negate { "^" } else { "" },
            self.inner
                .iter()
                .map(ClassType::to_string)
                .collect::<String>()
        )
    }
}
