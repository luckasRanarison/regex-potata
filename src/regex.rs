use crate::{
    error::Error,
    nfa::{Nfa, StateId, START as INITAL_STATE},
    parser::parse_regex,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

const FULL_CAPTURE_START: Option<usize> = Some(0);

type Captures = HashMap<usize, Vec<CaptureGroup>>;

#[derive(Debug)]
pub struct Regex {
    nfa: Nfa,
    start_capture: Captures,
    end_capture: Captures,
    capture_count: usize,
}

impl<'a> Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let ast = parse_regex(pattern)?;
        let nfa = Nfa::from(ast);
        let capture_count = nfa
            .capture_groups()
            .iter()
            .filter(|c| c.name.is_none())
            .count();
        let mut start_capture: Captures = HashMap::new();
        let mut end_capture: Captures = HashMap::new();

        for (index, group) in nfa.capture_groups().iter().enumerate() {
            start_capture
                .entry(group.start)
                .or_default()
                .push(CaptureGroup::new(index, group.name.clone()));
            end_capture
                .entry(group.end)
                .or_default()
                .push(CaptureGroup::new(index, group.name.clone()));
        }

        Ok(Self {
            nfa,
            start_capture,
            end_capture,
            capture_count,
        })
    }

    pub fn captures(&self, input: &'a str) -> Option<Capture<'a>> {
        let mut captures = vec![(None, None); self.capture_count];
        let mut named_captures = HashMap::new(); // TODO
        let mut end = None;
        let mut states = HashSet::new();

        states.insert(INITAL_STATE);

        for (i, ch) in input.char_indices() {
            states = states
                .iter()
                .flat_map(|&s| self.nfa.epsilon_closure(s))
                .collect();

            self.update_captures(&mut captures, &mut named_captures, &states, i);

            if self.has_accepting_state(&states) {
                end = Some(i)
            }

            states = states
                .iter()
                .flat_map(|state| self.nfa.next(*state, ch))
                .collect();

            if states.is_empty() {
                break;
            }
        }

        states = states
            .iter()
            .flat_map(|&s| self.nfa.epsilon_closure(s))
            .collect();

        self.update_captures(&mut captures, &mut named_captures, &states, input.len());

        if self.has_accepting_state(&states) {
            end = Some(input.len());
        }

        captures.insert(0, (FULL_CAPTURE_START, end));

        let groups = captures
            .into_iter()
            .flat_map(|(start, end)| start.and_then(|s| end.map(|e| (s, e))))
            .map(|(start, end)| Match::new(start, end, &input[start..end]))
            .collect::<Vec<_>>();

        if !groups.is_empty() {
            Some(Capture {
                groups,
                named_groups: HashMap::new(),
            })
        } else {
            None
        }
    }

    pub fn matches(&self, input: &'a str) -> Vec<Match<'a>> {
        let mut result = Vec::new();

        for (i, _) in input.char_indices() {
            let mut end = None;
            let mut states = HashSet::new();

            states.insert(INITAL_STATE);

            for (j, ch) in input[i..].char_indices() {
                states = states
                    .iter()
                    .flat_map(|&s| self.nfa.epsilon_closure(s))
                    .flat_map(|state| self.nfa.next(state, ch))
                    .flat_map(|s| self.nfa.epsilon_closure(s))
                    .collect();

                if self.has_accepting_state(&states) {
                    end = Some(i + j)
                }

                if states.is_empty() {
                    break;
                }
            }

            if let Some(end) = end {
                result.push(Match::new(i, end, &input[i..=end]))
            }
        }

        result
    }

    pub fn test(&self, input: &str) -> bool {
        !self.matches(input).is_empty()
    }

    fn has_accepting_state(&self, states: &HashSet<StateId>) -> bool {
        states.iter().any(|s| self.nfa.is_accepting(*s))
    }

    fn update_captures(
        &self,
        captures: &mut Vec<(Option<usize>, Option<usize>)>,
        named_captures: &mut HashMap<String, (Option<usize>, Option<usize>)>,
        states: &HashSet<StateId>,
        index: usize,
    ) {
        for state in states {
            if let Some(groups) = self.start_capture.get(&state) {
                for group in groups {
                    captures[group.index].0 = Some(index);
                }
            }
            if let Some(groups) = self.end_capture.get(&state) {
                for group in groups {
                    captures[group.index].1 = Some(index);
                }
            }
        }
    }
}

#[derive(Debug)]
struct CaptureGroup {
    index: usize,
    name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Capture<'a> {
    groups: Vec<Match<'a>>,
    named_groups: HashMap<String, Match<'a>>,
}

impl<'a> Capture<'a> {
    pub fn get(&self, index: usize) -> Option<&Match<'a>> {
        self.groups.get(index)
    }

    pub fn get_name(&self, name: &str) -> Option<&Match<'a>> {
        self.named_groups.get(name)
    }
}

impl CaptureGroup {
    fn new(index: usize, name: Option<String>) -> Self {
        Self { index, name }
    }
}

#[derive(Debug, PartialEq)]
pub struct Match<'a> {
    pub start: usize,
    pub end: usize,
    pub string: &'a str,
}

impl fmt::Display for Match<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl<'a> Match<'a> {
    fn new(start: usize, end: usize, string: &'a str) -> Self {
        Self { start, end, string }
    }
}

#[cfg(test)]
mod test {
    use crate::regex::Regex;

    #[test]
    fn test_simple_match() {
        let re = Regex::new("(mega|kilo)?bytes?").unwrap();

        assert!(re.test("byte"));
        assert!(re.test("bytes"));
        assert!(re.test("kilobyte"));
        assert!(re.test("kilobytes"));
        assert!(re.test("megabyte"));
        assert!(re.test("megabytes"));
    }

    #[test]
    fn test_plus_quantifier() {
        let re = Regex::new("eh+").unwrap();

        assert!(re.test("eh"));
        assert!(re.test("ehh"));
        assert!(re.test("ehhh"));
    }

    #[test]
    fn test_star_quantifier() {
        let re = Regex::new("n.*").unwrap();

        assert!(re.test("no"));
        assert!(re.test("nooo"));
        assert!(re.test("nooope"));
    }

    #[test]
    fn test_range_quantifier_simple() {
        let re = Regex::new("e{3}").unwrap();

        assert!(re.test("eee"));
        assert!(!re.test("e"));
        assert!(!re.test("ee"));

        let re = Regex::new("e{1,3}").unwrap();

        assert!(re.test("e"));
        assert!(re.test("ee"));
        assert!(re.test("eee"));
        assert!(!re.test(""));

        let re = Regex::new("e{3,}").unwrap();

        assert!(re.test("eee"));
        assert!(re.test("eeee"));
        assert!(re.test("eeeee"));
        assert!(!re.test(""));
        assert!(!re.test("e"));
        assert!(!re.test("ee"));
    }

    #[test]
    fn test_range_quantifier_extended() {
        let re = Regex::new("(h(ey|i)!?){2,}").unwrap();

        assert!(re.test("hihi"));
        assert!(re.test("hihi!hi"));
        assert!(re.test("heyhey!"));
        assert!(re.test("hey!hi"));
        assert!(!re.test(""));
        assert!(!re.test("hey!"));
    }

    #[test]
    fn test_character_class() {
        let re = Regex::new(r#"[0-9]+(\.[0-9]+)?"#).unwrap();

        assert!(re.test("10"));
        assert!(re.test("12.50"));
        assert!(!re.test(""));
    }
}
