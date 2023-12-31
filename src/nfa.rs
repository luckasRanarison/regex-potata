use crate::ast::Node;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    fmt::{self, Debug},
};

const START: usize = 0;

type StateId = usize;

#[derive(PartialEq)]
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

#[derive(PartialEq)]
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

#[derive(PartialEq)]
pub struct Nfa {
    state_count: usize,
    transitions: BTreeMap<usize, Vec<Transition>>,
}

impl Nfa {
    fn new(state_count: usize) -> Self {
        Self {
            state_count,
            transitions: BTreeMap::new(),
        }
    }

    fn end(&self) -> usize {
        self.state_count - 1
    }

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

    fn with_transition(mut self, from: StateId, transition: TransitionKind, to: StateId) -> Self {
        self.add_transition(from, transition, to);
        self
    }

    fn extend_transition(
        mut self,
        transitions: BTreeMap<usize, Vec<Transition>>,
        offset: usize,
    ) -> Self {
        for (start, transitions) in transitions {
            for transition in transitions {
                self.add_transition(start + offset, transition.kind, transition.end + offset);
            }
        }

        self
    }

    fn epsilon() -> Self {
        Nfa::new(2).with_transition(START, TransitionKind::Epsilon, 1)
    }

    fn character(ch: char) -> Self {
        Nfa::new(2).with_transition(START, TransitionKind::Char(ch), 1)
    }

    fn wildcard() -> Self {
        Nfa::new(2).with_transition(START, TransitionKind::Wildcard, 1)
    }

    fn concatenate(self, other: Nfa) -> Self {
        let offset = self.state_count;

        self.extend_transition(other.transitions, offset)
            .with_transition(offset - 1, TransitionKind::Epsilon, offset)
    }

    fn alternate(self, other: Nfa) -> Self {
        let offset = self.state_count + 1;
        let new_end = offset + other.state_count;

        Nfa::new(1)
            .with_transition(START, TransitionKind::Epsilon, 1)
            .with_transition(START, TransitionKind::Epsilon, offset)
            .extend_transition(self.transitions, 1)
            .extend_transition(other.transitions, offset)
            .with_transition(offset - 1, TransitionKind::Epsilon, new_end)
            .with_transition(new_end - 1, TransitionKind::Epsilon, new_end)
    }

    fn one_or_more(self) -> Self {
        let offset = self.state_count;

        Nfa::new(2)
            .with_transition(START, TransitionKind::Epsilon, 1)
            .extend_transition(self.transitions, 1)
            .with_transition(offset, TransitionKind::Epsilon, 1)
            .with_transition(offset, TransitionKind::Epsilon, offset + 1)
    }

    fn zero_or_one(self) -> Self {
        let end = self.end();
        self.with_transition(START, TransitionKind::Epsilon, end)
    }

    fn zero_or_more(self) -> Self {
        Nfa::zero_or_one(self).one_or_more()
    }

    fn epsilon_closure(&self, start: StateId) -> HashSet<StateId> {
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

    fn next_states(&self, state: StateId, input: char) -> HashSet<StateId> {
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

    pub fn test(&self, input: &str) -> bool {
        let mut states = HashSet::new();

        states.insert(START);

        for ch in input.chars() {
            states = states
                .iter()
                .flat_map(|&s| self.epsilon_closure(s))
                .flat_map(|state| self.next_states(state, ch))
                .collect();

            if states.is_empty() {
                return false;
            }
        }

        states = states
            .iter()
            .flat_map(|&s| self.epsilon_closure(s))
            .collect();

        states.contains(&self.end())
    }
}

impl From<Node> for Nfa {
    fn from(value: Node) -> Self {
        match value {
            Node::Empty => Nfa::epsilon(),
            Node::Character(ch) => Nfa::character(ch),
            Node::Concatenation(a, b) => {
                let a = Nfa::from(*a);
                let b = Nfa::from(*b);
                Nfa::concatenate(a, b)
            }
            Node::Alternation(a, b) => {
                let a = Nfa::from(*a);
                let b = Nfa::from(*b);
                Nfa::alternate(a, b)
            }
            Node::Plus(node) => {
                let n = Nfa::from(*node);
                Nfa::one_or_more(n)
            }
            Node::Star(node) => {
                let n = Nfa::from(*node);
                Nfa::zero_or_more(n)
            }
            Node::Optional(node) => {
                let n = Nfa::from(*node);
                Nfa::zero_or_one(n)
            }
            Node::Wildcard => Nfa::wildcard(),
            Node::Group(node) => Nfa::from(*node),
            Node::Range { inner, range } => todo!(),
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
    fn test_simple_match() {
        let nfa = to_nfa("(mega|kilo)?bytes?");

        assert!(nfa.test("byte"));
        assert!(nfa.test("bytes"));
        assert!(nfa.test("kilobyte"));
        assert!(nfa.test("kilobytes"));
        assert!(nfa.test("megabyte"));
        assert!(nfa.test("megabytes"));
    }

    #[test]
    fn test_plus_quantifiers() {
        let nfa = to_nfa("eh+");

        assert!(nfa.test("eh"));
        assert!(nfa.test("ehh"));
        assert!(nfa.test("ehhh"));
        assert!(!nfa.test("ehs"));
        assert!(!nfa.test("ehss"));
    }

    #[test]
    fn test_star_quantifiers() {
        let nfa = to_nfa("n.*");

        assert!(nfa.test("no"));
        assert!(nfa.test("nooo"));
        assert!(nfa.test("nooope"));
    }
}
