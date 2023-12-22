use std::{collections::HashSet, ops::RangeInclusive};

#[derive(Debug, PartialEq)]
pub enum RangeType {
    Exact(usize),
    Between(usize, usize),
    AtLeast(usize),
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Empty,
    Alternation(Box<Node>, Box<Node>),
    Concatenation(Box<Node>, Box<Node>),
    Star(Box<Node>),
    Plus(Box<Node>),
    Optional(Box<Node>),
    Range { inner: Box<Node>, range: RangeType },
    Group(Box<Node>),
    Wildcard,
    Character(char),
    // CharacterClass,
}
