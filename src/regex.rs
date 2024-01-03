use crate::{
    error::Error,
    nfa::{Nfa, START},
    parser::parse_regex,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Regex {
    nfa: Nfa,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let ast = parse_regex(pattern)?;
        let nfa = Nfa::from(ast);

        Ok(Self { nfa })
    }

    pub fn test(&self, input: &str) -> bool {
        let mut states = HashSet::new();

        states.insert(START);

        for ch in input.chars() {
            states = states
                .iter()
                .flat_map(|&s| self.nfa.epsilon_closure(s))
                .flat_map(|state| self.nfa.next(state, ch))
                .collect();

            if states.is_empty() {
                return false;
            }
        }

        states
            .iter()
            .flat_map(|&s| self.nfa.epsilon_closure(s))
            .any(|s| self.nfa.is_accepting(s))
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
        assert!(!re.test("ehs"));
        assert!(!re.test("ehss"));
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
        assert!(!re.test("ee"));
        assert!(!re.test("eeee"));

        let re = Regex::new("e{1,3}").unwrap();

        assert!(re.test("e"));
        assert!(re.test("ee"));
        assert!(re.test("eee"));
        assert!(!re.test(""));
        assert!(!re.test("eeee"));

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
