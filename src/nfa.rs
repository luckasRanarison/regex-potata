use crate::ast::{CharacterClass, ClassMember, Group, Node, Range};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::{self, Debug},
};

pub const START: usize = 0;

pub type StateId = usize;
pub type TransitionMap = BTreeMap<usize, Vec<Transition>>;

#[derive(Clone, PartialEq)]
pub enum TransitionKind {
    Character(char),
    Epsilon,
    Wildcard,
    CharacterClass(CharacterClass),
}

impl fmt::Display for TransitionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransitionKind::Epsilon => write!(f, "ε"),
            TransitionKind::Wildcard => write!(f, "wildcard"),
            TransitionKind::Character(ch) => match ch.is_whitespace() {
                true => write!(f, "whitespace"),
                false => write!(f, "{ch}"),
            },
            TransitionKind::CharacterClass(class) => write!(f, "{class}"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Transition {
    pub(crate) kind: TransitionKind,
    pub(crate) end: StateId,
}

impl Transition {
    fn new(kind: TransitionKind, end: StateId) -> Self {
        Self { kind, end }
    }

    fn is_epsilon(&self) -> bool {
        self.kind == TransitionKind::Epsilon
    }

    fn accept(&self, input: &char) -> bool {
        match &self.kind {
            TransitionKind::Character(ch) => ch == input,
            TransitionKind::Wildcard => true,
            TransitionKind::Epsilon => false,
            TransitionKind::CharacterClass(class) => {
                let contains = class.members.iter().any(|c| match c {
                    ClassMember::Atom(ch) => input == ch,
                    ClassMember::Range(lower, upper) => lower <= input && upper >= input,
                });

                class.negate ^ contains
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureGroup {
    pub start: StateId,
    pub end: StateId,
}

#[derive(Clone, PartialEq)]
pub struct Nfa {
    pub(crate) state_count: usize,
    pub(crate) transitions: TransitionMap,
    pub(crate) capture_groups: Vec<CaptureGroup>,
    pub(crate) named_capture_groups: HashMap<String, CaptureGroup>,
}

impl Nfa {
    fn end(&self) -> usize {
        self.state_count - 1
    }

    fn epsilon() -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .build()
    }

    fn character(ch: char) -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::Character(ch), 1)
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
            .extend(other, offset)
            .transition(offset - 1, TransitionKind::Epsilon, offset)
            .build()
    }

    fn alternate(self, other: Nfa) -> Self {
        let offset = self.state_count + 1;
        let new_end = offset + other.state_count;

        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .transition(START, TransitionKind::Epsilon, offset)
            .extend(self, 1)
            .extend(other, offset)
            .transition(offset - 1, TransitionKind::Epsilon, new_end)
            .transition(new_end - 1, TransitionKind::Epsilon, new_end)
            .build()
    }

    fn one_or_more(self) -> Self {
        let offset = self.state_count;

        NfaBuilder::default()
            .transition(START, TransitionKind::Epsilon, 1)
            .extend(self, 1)
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

    fn group(group: Group) -> Self {
        let nfa = Nfa::from(*group.inner);
        let end = nfa.end();

        match group.name {
            Some(name) => NfaBuilder::from(nfa).named_group(START, end, name).build(),
            None if group.is_capturing => NfaBuilder::from(nfa).group(START, end).build(),
            None => nfa,
        }
    }

    fn class(class: CharacterClass) -> Self {
        NfaBuilder::default()
            .transition(START, TransitionKind::CharacterClass(class), 1)
            .build()
    }

    pub fn epsilon_closure(&self, start: StateId) -> HashSet<StateId> {
        let mut eclosure = HashSet::new();
        let mut stack = VecDeque::new();

        stack.push_back(start);

        while let Some(state) = stack.pop_back() {
            if !eclosure.insert(state) {
                continue;
            }

            if let Some(transitions) = self.transitions.get(&state) {
                let eclosed_states = transitions
                    .iter()
                    .filter_map(|t| t.is_epsilon().then_some(t.end));
                stack.extend(eclosed_states);
            }
        }

        eclosure
    }

    pub fn next(&self, state: StateId, input: char) -> HashSet<StateId> {
        self.transitions
            .get(&state)
            .map_or_else(HashSet::new, |transitions| {
                transitions
                    .iter()
                    .filter_map(|t| t.accept(&input).then_some(t.end))
                    .collect()
            })
    }

    pub fn is_accepting(&self, state: StateId) -> bool {
        self.end() == state
    }
}

impl From<Node> for Nfa {
    fn from(value: Node) -> Self {
        match value {
            Node::Empty => Nfa::epsilon(),
            Node::Character(ch) => Nfa::character(ch),
            Node::Wildcard => Nfa::wildcard(),
            Node::Group(group) => Nfa::group(group),
            Node::Plus(node) => Nfa::from(*node).one_or_more(),
            Node::Star(node) => Nfa::from(*node).zero_or_more(),
            Node::Optional(node) => Nfa::from(*node).zero_or_one(),
            Node::Concatenation(a, b) => Nfa::from(*a).concatenate(Nfa::from(*b)),
            Node::Alternation(a, b) => Nfa::from(*a).alternate(Nfa::from(*b)),
            Node::Range { inner, range } => Nfa::from(*inner).range(range),
            Node::CharacterClass(class) => Nfa::class(class),
        }
    }
}

impl Debug for Nfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "State count: {:?}", self.state_count)?;
        writeln!(f, "Groups: {:?}", self.capture_groups)?;
        writeln!(f, "Transitions:")?;

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
    capture_groups: Vec<CaptureGroup>,
    named_capture_groups: HashMap<String, CaptureGroup>,
}

impl NfaBuilder {
    fn add_transition(&mut self, from: StateId, transition: TransitionKind, to: StateId) {
        let transition = Transition::new(transition, to);
        let transitions = self.transitions.entry(from).or_default();

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

    fn extend(mut self, other: Nfa, offset: usize) -> Self {
        for (start, transitions) in other.transitions {
            for transition in transitions {
                self.add_transition(start + offset, transition.kind, transition.end + offset);
            }
        }

        for group in other.capture_groups {
            self.capture_groups.push(CaptureGroup {
                start: group.start + offset,
                end: group.end + offset,
            });
        }

        for (name, group) in other.named_capture_groups {
            self.named_capture_groups.insert(
                name,
                CaptureGroup {
                    start: group.start + offset,
                    end: group.end + offset,
                },
            );
        }

        self
    }

    fn group(mut self, start: StateId, end: StateId) -> Self {
        self.capture_groups.insert(0, CaptureGroup { start, end });
        self
    }

    fn named_group(mut self, start: StateId, end: StateId, name: String) -> Self {
        self.named_capture_groups
            .insert(name, CaptureGroup { start, end });
        self
    }

    fn build(self) -> Nfa {
        Nfa {
            state_count: self.state_count,
            transitions: self.transitions,
            capture_groups: self.capture_groups,
            named_capture_groups: self.named_capture_groups,
        }
    }
}

impl From<Nfa> for NfaBuilder {
    fn from(value: Nfa) -> Self {
        Self {
            state_count: value.state_count,
            transitions: value.transitions,
            capture_groups: value.capture_groups,
            named_capture_groups: value.named_capture_groups,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_regex;

    fn to_nfa(regex: &str) -> Nfa {
        let ast = parse_regex(regex).unwrap();
        Nfa::from(ast)
    }

    #[test]
    fn test_concatenation() {
        let expected = NfaBuilder::default()
            .transition(0, TransitionKind::Character('h'), 1)
            .transition(1, TransitionKind::Epsilon, 2)
            .transition(2, TransitionKind::Character('i'), 3)
            .build();
        let nfa = to_nfa("hi");

        assert_eq!(expected, nfa);
    }

    #[test]
    fn test_alternation() {
        let expected = NfaBuilder::default()
            .transition(0, TransitionKind::Epsilon, 1)
            .transition(0, TransitionKind::Epsilon, 3)
            .transition(1, TransitionKind::Character('a'), 2)
            .transition(2, TransitionKind::Epsilon, 5)
            .transition(3, TransitionKind::Character('b'), 4)
            .transition(4, TransitionKind::Epsilon, 5)
            .build();
        let nfa = to_nfa("a|b");

        assert_eq!(expected, nfa);
    }

    #[test]
    fn test_range_excat() {
        let expected = NfaBuilder::default()
            .transition(0, TransitionKind::Character('e'), 1)
            .transition(1, TransitionKind::Epsilon, 2)
            .transition(2, TransitionKind::Character('e'), 3)
            .transition(3, TransitionKind::Epsilon, 4)
            .transition(4, TransitionKind::Character('e'), 5)
            .build();
        let nfa = to_nfa("e{3}");

        assert_eq!(expected, nfa);
    }

    #[test]
    fn test_range_between() {
        let expected = NfaBuilder::default()
            .transition(0, TransitionKind::Character('e'), 1)
            .transition(1, TransitionKind::Epsilon, 2)
            .transition(2, TransitionKind::Character('e'), 3)
            .transition(2, TransitionKind::Epsilon, 3)
            .build();
        let nfa = to_nfa("e{1,2}");

        assert_eq!(expected, nfa);
    }

    #[test]
    fn test_range_minimum() {
        let expected = NfaBuilder::default()
            .transition(0, TransitionKind::Character('e'), 1)
            .transition(1, TransitionKind::Epsilon, 2)
            .transition(2, TransitionKind::Character('e'), 3)
            .transition(3, TransitionKind::Epsilon, 1)
            .transition(3, TransitionKind::Epsilon, 4)
            .build();
        let nfa = to_nfa("e{2,}");

        assert_eq!(expected, nfa);
    }

    #[test]
    fn test_epsilon_closure() {
        let nfa = NfaBuilder::default()
            .transition(0, TransitionKind::Epsilon, 1)
            .transition(0, TransitionKind::Epsilon, 2)
            .transition(1, TransitionKind::Epsilon, 3)
            .build();
        let expected = [0, 1, 2, 3].into_iter().collect();
        let eclosure = nfa.epsilon_closure(0);

        assert_eq!(eclosure, expected);

        let nfa = NfaBuilder::default()
            .transition(0, TransitionKind::Character('a'), 1)
            .build();
        let eclosure = nfa.epsilon_closure(0);
        let expected = [0].into_iter().collect();

        assert_eq!(eclosure, expected);

        let nfa = NfaBuilder::default()
            .transition(0, TransitionKind::Epsilon, 1)
            .transition(1, TransitionKind::Epsilon, 2)
            .transition(2, TransitionKind::Epsilon, 1)
            .build();
        let expected = [0, 1, 2].into_iter().collect();
        let eclosure = nfa.epsilon_closure(0);

        assert_eq!(eclosure, expected);
    }

    #[test]
    fn test_capture_group_order() {
        let nfa = to_nfa("a(b(c)(d))(e)");
        let expected = vec![
            CaptureGroup { start: 2, end: 7 },
            CaptureGroup { start: 4, end: 5 },
            CaptureGroup { start: 6, end: 7 },
            CaptureGroup { start: 8, end: 9 },
        ];

        assert_eq!(nfa.capture_groups, expected);
    }
}
