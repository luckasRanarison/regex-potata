use std::{collections::HashSet, ops::RangeInclusive};

#[derive(Debug, PartialEq)]
pub enum Node {
    Empty,
    Alternation(Box<Node>, Box<Node>),
    Concatenation(Box<Node>, Box<Node>),
    Star(Box<Node>),
    Plus(Box<Node>),
    Optional(Box<Node>),
    RangeQuantifier(Box<Node>, RangeInclusive<usize>),
    Group(Box<Node>),
    Wildcard,
    Char(char),
    // CharClass,
}
