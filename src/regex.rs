use crate::{
    error::Error,
    nfa::{Nfa, StateId, START as INITAL_STATE},
    parser::parse_regex,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};

type Captures = HashMap<usize, Vec<CaptureKind>>;

#[derive(Debug)]
pub struct Regex {
    pub(crate) nfa: Nfa,
    start_capture: Captures,
    end_capture: Captures,
}

impl<'a> Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let ast = parse_regex(pattern)?;
        let nfa = Nfa::from(ast);
        let mut start_capture: Captures = HashMap::new();
        let mut end_capture: Captures = HashMap::new();

        for (index, group) in nfa.capture_groups.iter().enumerate() {
            start_capture
                .entry(group.start)
                .or_default()
                .push(CaptureKind::Indexed(index));
            end_capture
                .entry(group.end)
                .or_default()
                .push(CaptureKind::Indexed(index));
        }

        for (name, group) in nfa.named_capture_groups.iter() {
            start_capture
                .entry(group.start)
                .or_default()
                .push(CaptureKind::Named(name.to_string()));
            end_capture
                .entry(group.end)
                .or_default()
                .push(CaptureKind::Named(name.to_string()));
        }

        Ok(Self {
            nfa,
            start_capture,
            end_capture,
        })
    }

    pub fn captures(&self, input: &'a str) -> Option<Capture<'a>> {
        self.matches(input, false).into_iter().next()
    }

    pub fn captures_all(&self, input: &'a str) -> Vec<Capture<'a>> {
        self.matches(input, true)
    }

    pub fn find(&self, input: &'a str) -> Option<Match<'a>> {
        self.matches(input, false)
            .into_iter()
            .next()
            .and_then(|mut c| c.captures.remove(&0))
    }

    pub fn find_all(&self, input: &'a str) -> Vec<Match<'a>> {
        self.matches(input, true)
            .into_iter()
            .flat_map(|mut captures| captures.captures.remove(&0))
            .collect()
    }

    fn matches(&self, input: &'a str, all: bool) -> Vec<Capture<'a>> {
        let mut result: Vec<Capture<'_>> = Vec::new();

        for (i, _) in input.char_indices() {
            let start = match result.last() {
                Some(capt) => capt.get(0).map_or(i, |m| if i > m.end { i } else { m.end }),
                None => i,
            };

            if i < start {
                continue;
            }

            let input_len = input[start..].len();
            let mut char_count = 0;
            let mut captures = HashMap::new();
            let mut named_captures = HashMap::new();
            let mut states = HashSet::new();
            let mut end = None;
            let mut char_index_map = HashMap::new();

            states.insert(INITAL_STATE);

            for (j, ch) in input[start..].char_indices() {
                char_index_map.insert(j + start, char_count);
                char_count += 1;

                states = states
                    .iter()
                    .flat_map(|&s| self.nfa.epsilon_closure(s))
                    .collect();

                self.update_captures(&mut captures, &mut named_captures, &states, j + start);

                if self.has_accepting_state(&states) {
                    end = Some(j + start)
                }

                states = states
                    .iter()
                    .flat_map(|state| self.nfa.next(*state, ch))
                    .collect();

                if states.is_empty() {
                    break;
                }
            }

            char_index_map.insert(input_len, char_count);
            states = states
                .iter()
                .flat_map(|&s| self.nfa.epsilon_closure(s))
                .collect();

            self.update_captures(
                &mut captures,
                &mut named_captures,
                &states,
                input_len + start,
            );

            if self.has_accepting_state(&states) {
                end = Some(input_len + start);
            }

            if end.is_none() {
                continue;
            }

            let captures = captures
                .into_iter()
                .chain([(0, (Some(start), end))])
                .flat_map(|(index, (start, end))| Some(index).zip(self.new_mach(input, start, end)))
                .collect();
            let named_captures = named_captures
                .into_iter()
                .flat_map(|(name, (start, end))| Some(name).zip(self.new_mach(input, start, end)))
                .collect();

            let capture = Capture {
                captures,
                named_captures,
            };

            if !all {
                return vec![capture];
            }

            result.push(capture)
        }

        result
    }

    pub fn test(&self, input: &str) -> bool {
        self.find(input).is_some()
    }

    fn has_accepting_state(&self, states: &HashSet<StateId>) -> bool {
        states.iter().any(|s| self.nfa.is_accepting(*s))
    }

    fn new_mach(
        &self,
        input: &'a str,
        start: Option<usize>,
        end: Option<usize>,
    ) -> Option<Match<'a>> {
        start
            .zip(end)
            .map(|(start, end)| Match::new(start, end, &input[start..end]))
    }

    fn update_captures(
        &self,
        captures: &mut HashMap<usize, (Option<usize>, Option<usize>)>,
        named_captures: &mut HashMap<String, (Option<usize>, Option<usize>)>,
        states: &HashSet<StateId>,
        position: usize,
    ) {
        for state in states {
            if let Some(groups) = self.start_capture.get(state) {
                for group in groups {
                    self.update_capture_start(captures, named_captures, group, position);
                }
            }
            if let Some(groups) = self.end_capture.get(state) {
                for group in groups {
                    self.update_capture_end(captures, named_captures, group, position);
                }
            }
        }
    }

    fn update_capture_start(
        &self,
        captures: &mut HashMap<usize, (Option<usize>, Option<usize>)>,
        named_captures: &mut HashMap<String, (Option<usize>, Option<usize>)>,
        group: &CaptureKind,
        position: usize,
    ) {
        match group {
            CaptureKind::Indexed(index) => {
                captures
                    .entry(*index + 1)
                    .and_modify(|(start, end)| {
                        if end.is_none() {
                            *start = Some(position)
                        }
                    })
                    .or_insert((Some(position), None));
            }
            CaptureKind::Named(name) => {
                named_captures
                    .entry(name.to_owned())
                    .and_modify(|(start, end)| {
                        if end.is_none() {
                            *start = Some(position)
                        }
                    })
                    .or_insert((Some(position), None));
            }
        }
    }

    fn update_capture_end(
        &self,
        captures: &mut HashMap<usize, (Option<usize>, Option<usize>)>,
        named_captures: &mut HashMap<String, (Option<usize>, Option<usize>)>,
        group: &CaptureKind,
        position: usize,
    ) {
        match group {
            CaptureKind::Indexed(index) => {
                captures
                    .entry(*index + 1)
                    .and_modify(|(_, end)| *end = Some(position));
            }
            CaptureKind::Named(name) => {
                named_captures
                    .entry(name.to_owned())
                    .and_modify(|(_, end)| *end = Some(position));
            }
        }
    }
}

#[derive(Debug)]
enum CaptureKind {
    Indexed(usize),
    Named(String),
}

#[derive(Debug, PartialEq)]
pub struct Capture<'a> {
    pub(crate) captures: BTreeMap<usize, Match<'a>>,
    pub(crate) named_captures: HashMap<String, Match<'a>>,
}

impl<'a> Capture<'a> {
    pub fn get(&self, index: usize) -> Option<&Match<'a>> {
        self.captures.get(&index)
    }

    pub fn get_name(&self, name: &str) -> Option<&Match<'a>> {
        self.named_captures.get(name)
    }

    pub fn iter(&'a self) -> CaptureIterator<'a> {
        CaptureIterator::new(self)
    }
}

pub struct CaptureIterator<'a> {
    capture: &'a Capture<'a>,
    current_index: usize,
}

impl<'a> CaptureIterator<'a> {
    pub fn new(capture: &'a Capture<'a>) -> Self {
        CaptureIterator {
            capture,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for CaptureIterator<'a> {
    type Item = &'a Match<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(match_item) = self.capture.get(self.current_index) {
            self.current_index += 1;
            Some(match_item)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Capture<'a> {
    type Item = &'a Match<'a>;
    type IntoIter = CaptureIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CaptureIterator::new(self)
    }
}

impl<'a> IntoIterator for Capture<'a> {
    type Item = Match<'a>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.captures.into_values())
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
    // use std::collections::HashMap;

    use crate::{
        regex::{Match, Regex},
        // Capture,
    };

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

    #[test]
    fn test_capture_groups() {
        let regex = Regex::new(r#"(ah+)(:?eh+)(oh+)"#).unwrap();
        let matches = regex.captures("ahhhhehhhohhh").unwrap();

        assert_eq!(matches.get(0), Some(&Match::new(0, 13, "ahhhhehhhohhh")));
        assert_eq!(matches.get(1), Some(&Match::new(0, 5, "ahhhh")));
        assert_eq!(matches.get(2), Some(&Match::new(9, 13, "ohhh")));
    }

    #[test]
    fn test_named_capture_groups() {
        let regex = Regex::new(r#"(?<hour>\d+):(?<minute>\d+)"#).unwrap();
        let matches = regex.captures("19:30").unwrap();

        assert_eq!(matches.get(0), Some(&Match::new(0, 5, "19:30")));
        assert_eq!(matches.get_name("hour"), Some(&Match::new(0, 2, "19")));
        assert_eq!(matches.get_name("minute"), Some(&Match::new(3, 5, "30")));
    }

    // #[test]
    // fn test_repeated_group() {
    //     let regex = Regex::new(r#"(hi)+(ah)+"#).unwrap();
    //     let matches = regex.captures("hihiah").unwrap();
    //     let expected = Capture {
    //         captures: vec![
    //             (0, Match::new(0, 6, "hihiah")),
    //             (1, Match::new(2, 4, "hi")),
    //             (2, Match::new(4, 6, "ah")),
    //         ]
    //         .into_iter()
    //         .collect(),
    //         named_captures: HashMap::new(),
    //     };
    //
    //     assert_eq!(matches, expected);
    // }

    #[test]
    fn test_find() {
        let regex = Regex::new(r#"wh(at|o|y)"#).unwrap();
        let matches = regex.find_all("what? who? why?");

        assert_eq!(
            matches,
            vec![
                Match::new(0, 4, "what"),
                Match::new(6, 9, "who"),
                Match::new(11, 14, "why")
            ]
        );
    }
}
