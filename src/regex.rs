use crate::{
    error::Error,
    nfa::{Nfa, START},
    parser::Parser,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Regex {
    nfa: Nfa,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let ast = Parser::new(pattern).parse()?;
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
            .any(|s| s == self.nfa.end())
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
    fn test_plus_quantifiers() {
        let nfa = Regex::new("eh+").unwrap();

        assert!(nfa.test("eh"));
        assert!(nfa.test("ehh"));
        assert!(nfa.test("ehhh"));
        assert!(!nfa.test("ehs"));
        assert!(!nfa.test("ehss"));
    }

    #[test]
    fn test_star_quantifiers() {
        let nfa = Regex::new("n.*").unwrap();

        assert!(nfa.test("no"));
        assert!(nfa.test("nooo"));
        assert!(nfa.test("nooope"));
    }
}
