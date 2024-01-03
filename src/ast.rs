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
    CharacterClass(Class),
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
pub enum ClassMember {
    Atom(char),
    Range(char, char),
}

impl fmt::Display for ClassMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassMember::Atom(ch) => write!(f, "{ch}"),
            ClassMember::Range(lower, upper) => write!(f, "{lower}-{upper}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub negate: bool,
    pub members: Vec<ClassMember>,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}{}]",
            if self.negate { "^" } else { "" },
            self.members
                .iter()
                .map(ClassMember::to_string)
                .collect::<String>()
        )
    }
}
