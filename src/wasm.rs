use crate::{
    nfa::{StateId, TransitionKind},
    regex::{Capture, Regex},
    Match,
};
use std::collections::{BTreeMap, HashMap};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RegexEngine {
    engine: Regex,
}

#[wasm_bindgen]
impl RegexEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(pattern: &str) -> Self {
        Self {
            engine: Regex::new(pattern).unwrap(),
        }
    }

    pub fn captures(&self, input: &str) -> Option<RegexCapture> {
        let index_map = get_char_index(input);

        self.engine
            .captures(input)
            .map(|c| RegexCapture::from_capture(c, &index_map))
    }

    pub fn find(&self, input: &str) -> Option<RegexMatch> {
        let index_map = get_char_index(input);

        self.engine
            .find(input)
            .map(|m| RegexMatch::from_match(m, &index_map))
    }

    #[wasm_bindgen(js_name = "findAll")]
    pub fn find_all(&self, input: &str) -> Vec<RegexMatch> {
        let index_map = get_char_index(input);

        self.engine
            .find_all(input)
            .into_iter()
            .map(|m| RegexMatch::from_match(m, &index_map))
            .collect()
    }

    pub fn test(&self, input: &str) -> bool {
        self.engine.test(input)
    }

    #[wasm_bindgen(js_name = "nfaStates")]
    pub fn nfa_states(&self) -> Vec<StateId> {
        (0..self.engine.nfa.state_count).collect()
    }

    #[wasm_bindgen(js_name = "nfaTransition")]
    pub fn nfa_transition(&self, from: StateId) -> Option<Vec<Transition>> {
        self.engine.nfa.transitions.get(&from).map(|transitions| {
            transitions
                .iter()
                .map(|t| Transition {
                    kind: t.kind.clone(),
                    end: t.end,
                })
                .collect()
        })
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
}

impl RegexMatch {
    fn from_match(value: Match<'_>, index_map: &HashMap<usize, usize>) -> Self {
        Self {
            start: index_map[&value.start],
            end: index_map[&value.end],
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RegexCapture {
    captures: BTreeMap<usize, RegexMatch>,
    named_captures: HashMap<String, RegexMatch>,
}

impl RegexCapture {
    fn from_capture(value: Capture, index_map: &HashMap<usize, usize>) -> Self {
        let captures = value
            .captures
            .into_iter()
            .map(|(i, v)| (i, RegexMatch::from_match(v, index_map)))
            .collect();
        let named_captures = value
            .named_captures
            .into_iter()
            .map(|(i, v)| (i, RegexMatch::from_match(v, index_map)))
            .collect();

        Self {
            captures,
            named_captures,
        }
    }
}

#[wasm_bindgen]
impl RegexCapture {
    pub fn get(&self, index: usize) -> Option<RegexMatch> {
        self.captures.get(&index).cloned()
    }

    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self, name: &str) -> Option<RegexMatch> {
        self.named_captures.get(name).cloned()
    }

    pub fn captures(&self) -> Vec<usize> {
        self.captures.keys().cloned().collect()
    }

    #[wasm_bindgen(js_name = "namedCaptures")]
    pub fn named_captures(&self) -> Vec<String> {
        self.named_captures.keys().cloned().collect()
    }
}

#[wasm_bindgen]
pub struct Transition {
    #[wasm_bindgen]
    pub end: StateId,
    kind: TransitionKind,
}

#[wasm_bindgen]
impl Transition {
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.kind.to_string()
    }
}

fn get_char_index(input: &str) -> HashMap<usize, usize> {
    input
        .char_indices()
        .enumerate()
        .map(|(char_idx, (slice_idex, _))| (slice_idex, char_idx))
        .chain([(input.len(), input.chars().count())])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{RegexEngine, RegexMatch};

    #[test]
    fn test_unicode_range() {
        let regex = RegexEngine::new(r#"ここ"#);
        let matches = regex.find_all("ここでここで");

        assert_eq!(
            matches,
            vec![
                RegexMatch { start: 0, end: 2 },
                RegexMatch { start: 3, end: 5 },
            ]
        );

        let regex = RegexEngine::new(r#"日本語"#);
        let matches = regex.find_all("これは日本語のテストです。日本語");

        assert_eq!(
            matches,
            vec![
                RegexMatch { start: 3, end: 6 },
                RegexMatch { start: 13, end: 16 },
            ]
        );
    }
}
