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
    Group(Group),
    Wildcard,
    Character(char),
    CharacterClass(CharacterClass),
}

impl Node {
    pub fn alternation(lhs: Node, rhs: Node) -> Self {
        Self::Alternation(Box::new(lhs), Box::new(rhs))
    }

    pub fn concatenation(lhs: Node, rhs: Node) -> Self {
        Self::Concatenation(Box::new(lhs), Box::new(rhs))
    }

    pub fn star(operand: Node) -> Self {
        Self::Star(Box::new(operand))
    }

    pub fn plus(operand: Node) -> Self {
        Self::Plus(Box::new(operand))
    }

    pub fn optional(operand: Node) -> Self {
        Self::Optional(Box::new(operand))
    }

    pub fn range(inner: Node, range: Range) -> Self {
        Self::Range {
            inner: Box::new(inner),
            range,
        }
    }

    pub fn group(inner: Node, is_capturing: bool, name: Option<&str>) -> Self {
        Self::Group(Group::new(inner, is_capturing, name.map(str::to_string)))
    }

    pub fn class(negate: bool, members: Vec<ClassMember>) -> Self {
        Self::CharacterClass(CharacterClass { negate, members })
    }
}

#[derive(Debug, PartialEq)]
pub struct Group {
    pub inner: Box<Node>,
    pub name: Option<String>,
    pub is_capturing: bool,
}

impl Group {
    fn new(inner: Node, is_capturing: bool, name: Option<String>) -> Self {
        Self {
            inner: Box::new(inner),
            name,
            is_capturing,
        }
    }
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
pub struct CharacterClass {
    pub negate: bool,
    pub members: Vec<ClassMember>,
}

impl fmt::Display for CharacterClass {
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
