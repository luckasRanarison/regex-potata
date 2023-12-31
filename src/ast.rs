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
    // CharacterClass,
}
