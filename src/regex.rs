use crate::{
    error::Error,
    nfa::{Nfa, StateId, START},
    parser::parse_regex,
};
use std::{collections::HashSet, fmt};

#[derive(Debug)]
pub struct Regex {
    nfa: Nfa,
}

impl<'a> Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let ast = parse_regex(pattern)?;
        let nfa = Nfa::from(ast);

        Ok(Self { nfa })
    }

    pub fn matches(&self, input: &'a str) -> Vec<Match<'a>> {
        let mut result = Vec::new();

        for (i, _) in input.char_indices() {
            let mut end = None;
            let mut states = HashSet::new();

            states.insert(START);

            for (j, ch) in input[i..].char_indices() {
                states = states
                    .iter()
                    .flat_map(|&s| self.nfa.epsilon_closure(s))
                    .flat_map(|state| self.nfa.next(state, ch))
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
        states
            .iter()
            .flat_map(|&s| self.nfa.epsilon_closure(s))
            .any(|s| self.nfa.is_accepting(s))
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
