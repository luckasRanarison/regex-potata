use crate::ast::{Node, Range};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    fmt::{self, Debug},
};

pub const START: usize = 0;

type StateId = usize;
type TransitionMap = BTreeMap<usize, Vec<Transition>>;

#[derive(Clone, PartialEq)]
enum TransitionKind {
    Char(char),
    Epsilon,
    Wildcard,
}

impl fmt::Display for TransitionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransitionKind::Char(ch) => write!(f, "{ch}"),
            TransitionKind::Epsilon => write!(f, "ε"),
            TransitionKind::Wildcard => write!(f, "."),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Transition {
    kind: TransitionKind,
    end: StateId,
}

impl Transition {
    fn new(kind: TransitionKind, end: StateId) -> Self {
        Self { kind, end }
    }

    fn is_epsilon(&self) -> bool {
        self.kind == TransitionKind::Epsilon
    }
}

#[derive(Clone, PartialEq)]
pub struct Nfa {
    state_count: usize,
    transitions: TransitionMap,
}

impl Nfa {
    pub fn end(&self) -> usize {
        self.state_count - 1
    }

    fn epsilon() -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .build()
    }

    fn character(ch: char) -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::Char(ch), 1)
            .build()
    }

    fn wildcard() -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::Wildcard, 1)
            .build()
    }

    fn concatenate(self, other: Nfa) -> Self {
        let offset = self.state_count;

        NfaBuilder::from(self)
            .extend(other.transitions, offset)
            .transition(offset - 1, TransitionKind::Epsilon, offset)
            .build()
    }

    fn alternate(self, other: Nfa) -> Self {
        let offset = self.state_count + 1;
        let new_end = offset + other.state_count;

        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .transition(START, TransitionKind::Epsilon, offset)
            .extend(self.transitions, 1)
            .extend(other.transitions, offset)
            .transition(offset - 1, TransitionKind::Epsilon, new_end)
            .transition(new_end - 1, TransitionKind::Epsilon, new_end)
            .build()
    }

    fn one_or_more(self) -> Self {
        let offset = self.state_count;

        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .extend(self.transitions, 1)
            .transition(offset, TransitionKind::Epsilon, 1)
            .transition(offset, TransitionKind::Epsilon, offset + 1)
            .build()
    }

    fn zero_or_one(self) -> Self {
        let end = self.end();

        NfaBuilder::from(self)
            .transition(START, TransitionKind::Epsilon, end)
            .build()
    }

    fn zero_or_more(self) -> Self {
        Nfa::zero_or_one(self).one_or_more()
    }

    fn range(self, range: Range) -> Self {
        let mut nfa = self;
        let clone = nfa.clone();

        for _ in 1..range.min {
            nfa = nfa.concatenate(clone.clone())
        }

        if let Some(max) = range.max {
            for _ in range.min..max {
                nfa = nfa.concatenate(clone.clone().zero_or_one())
            }

            nfa
        } else {
            let end = nfa.end();

            NfaBuilder::from(nfa)
                .transition(end, TransitionKind::Epsilon, end - clone.state_count)
                .transition(end, TransitionKind::Epsilon, end + 1)
                .build()
        }
    }

    pub fn epsilon_closure(&self, start: StateId) -> HashSet<StateId> {
        let mut eclosure = HashSet::new();
        let mut stack = VecDeque::new();

        stack.push_back(start);

        while let Some(state) = stack.pop_back() {
            if let Some(transitions) = self.transitions.get(&state) {
                let eclosed = transitions.iter().filter_map(|t| match t.is_epsilon() {
                    true if !eclosure.contains(&t.end) => Some(t.end),
                    _ => None,
                });

                stack.extend(eclosed);
            }

            eclosure.insert(state);
        }

        eclosure
    }

    pub fn next(&self, state: StateId, input: char) -> HashSet<StateId> {
        if let Some(transitions) = self.transitions.get(&state) {
            transitions
                .iter()
                .filter_map(|t| match t.kind {
                    TransitionKind::Char(ch) if ch == input => Some(t.end),
                    TransitionKind::Wildcard => Some(t.end),
                    _ => None,
                })
                .collect()
        } else {
            HashSet::new()
        }
    }
}

impl From<Node> for Nfa {
    fn from(value: Node) -> Self {
        match value {
            Node::Empty => Nfa::epsilon(),
            Node::Character(ch) => Nfa::character(ch),
            Node::Wildcard => Nfa::wildcard(),
            Node::Group(node) => Nfa::from(*node),
            Node::Plus(node) => Nfa::from(*node).one_or_more(),
            Node::Star(node) => Nfa::from(*node).zero_or_more(),
            Node::Optional(node) => Nfa::from(*node).zero_or_one(),
            Node::Concatenation(a, b) => Nfa::from(*a).concatenate(Nfa::from(*b)),
            Node::Alternation(a, b) => Nfa::from(*a).alternate(Nfa::from(*b)),
            Node::Range { inner, range } => Nfa::from(*inner).range(range),
        }
    }
}

impl Debug for Nfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "State count: {:?}", self.state_count)?;

        for (start, transitions) in &self.transitions {
            for transition in transitions {
                writeln!(f, "{} -> {} ({})", start, transition.end, transition.kind)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct NfaBuilder {
    state_count: usize,
    transitions: TransitionMap,
}

impl NfaBuilder {
    fn add_transition(&mut self, from: StateId, transition: TransitionKind, to: StateId) {
        let transition = Transition::new(transition, to);
        let transitions = self.transitions.entry(from).or_insert(vec![]);

        transitions.push(transition);

        if from + 1 > self.state_count {
            self.state_count += 1;
        }
        if to + 1 > self.state_count {
            self.state_count += 1;
        }
    }

    fn transition(mut self, from: StateId, transition: TransitionKind, to: StateId) -> Self {
        self.add_transition(from, transition, to);
        self
    }

    fn extend(mut self, transitions: BTreeMap<usize, Vec<Transition>>, offset: usize) -> Self {
        for (start, transitions) in transitions {
            for transition in transitions {
                self.add_transition(start + offset, transition.kind, transition.end + offset);
            }
        }

        self
    }

    fn build(self) -> Nfa {
        Nfa {
            state_count: self.state_count,
            transitions: self.transitions,
        }
    }
}

impl From<Nfa> for NfaBuilder {
    fn from(value: Nfa) -> Self {
        Self {
            state_count: value.state_count,
            transitions: value.transitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn to_nfa(regex: &str) -> Nfa {
        let ast = Parser::new(regex).parse().unwrap();
        Nfa::from(ast)
    }

    #[test]
    fn test_concatenation() {
        let result = to_nfa("hi");
        let transitions = vec![
            (0, vec![Transition::new(TransitionKind::Char('h'), 1)]),
            (1, vec![Transition::new(TransitionKind::Epsilon, 2)]),
            (2, vec![Transition::new(TransitionKind::Char('i'), 3)]),
        ];
        let expected = Nfa {
            state_count: 4,
            transitions: transitions.into_iter().collect(),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_alternation() {
        let result = to_nfa("a|b");
        let transitions = vec![
            (
                0,
                vec![
                    Transition::new(TransitionKind::Epsilon, 1),
                    Transition::new(TransitionKind::Epsilon, 3),
                ],
            ),
            (1, vec![Transition::new(TransitionKind::Char('a'), 2)]),
            (2, vec![Transition::new(TransitionKind::Epsilon, 5)]),
            (3, vec![Transition::new(TransitionKind::Char('b'), 4)]),
            (4, vec![Transition::new(TransitionKind::Epsilon, 5)]),
        ];
        let expected = Nfa {
            state_count: 6,
            transitions: transitions.into_iter().collect(),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_range_excat() {
        let nfa = to_nfa("e{3}");
        let transitions = vec![
            (0, vec![Transition::new(TransitionKind::Char('e'), 1)]),
            (1, vec![Transition::new(TransitionKind::Epsilon, 2)]),
            (2, vec![Transition::new(TransitionKind::Char('e'), 3)]),
            (3, vec![Transition::new(TransitionKind::Epsilon, 4)]),
            (4, vec![Transition::new(TransitionKind::Char('e'), 5)]),
        ];
        let expected = Nfa {
            state_count: 6,
            transitions: transitions.into_iter().collect(),
        };

        assert_eq!(nfa, expected);
    }

    #[test]
    fn test_range_between() {
        let nfa = to_nfa("e{1,2}");
        let transitions = vec![
            (0, vec![Transition::new(TransitionKind::Char('e'), 1)]),
            (1, vec![Transition::new(TransitionKind::Epsilon, 2)]),
            (
                2,
                vec![
                    Transition::new(TransitionKind::Char('e'), 3),
                    Transition::new(TransitionKind::Epsilon, 3),
                ],
            ),
        ];
        let expected = Nfa {
            state_count: 4,
            transitions: transitions.into_iter().collect(),
        };

        assert_eq!(nfa, expected);
    }

    #[test]
    fn test_range_minimum() {
        let nfa = to_nfa("e{2,}");
        let transitions = vec![
            (0, vec![Transition::new(TransitionKind::Char('e'), 1)]),
            (1, vec![Transition::new(TransitionKind::Epsilon, 2)]),
            (2, vec![Transition::new(TransitionKind::Char('e'), 3)]),
            (
                3,
                vec![
                    Transition::new(TransitionKind::Epsilon, 1),
                    Transition::new(TransitionKind::Epsilon, 4),
                ],
            ),
        ];
        let expected = Nfa {
            state_count: 5,
            transitions: transitions.into_iter().collect(),
        };

        assert_eq!(nfa, expected);
    }
}
