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

    pub fn captures(&self, input: &str) -> Option<OwnedCapture> {
        let index_map = get_char_index(input);

        self.engine
            .captures(input)
            .map(|c| OwnedCapture::from_capture(c, &index_map))
    }

    pub fn find(&self, input: &str) -> Option<OwnedMatch> {
        let index_map = get_char_index(input);

        self.engine
            .find(input)
            .map(|m| OwnedMatch::from_match(m, &index_map))
    }

    #[wasm_bindgen(js_name = "findAll")]
    pub fn find_all(&self, input: &str) -> Vec<OwnedMatch> {
        let index_map = get_char_index(input);

        self.engine
            .find_all(input)
            .into_iter()
            .map(|m| OwnedMatch::from_match(m, &index_map))
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
pub struct OwnedMatch {
    pub start: usize,
    pub end: usize,
}

impl OwnedMatch {
    fn from_match(value: Match<'_>, index_map: &HashMap<usize, usize>) -> Self {
        Self {
            start: index_map[&value.start],
            end: index_map[&value.end],
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct OwnedCapture {
    captures: BTreeMap<usize, OwnedMatch>,
    named_captures: HashMap<String, OwnedMatch>,
}

impl OwnedCapture {
    fn from_capture(value: Capture, index_map: &HashMap<usize, usize>) -> Self {
        let captures = value
            .captures
            .into_iter()
            .map(|(i, v)| (i, OwnedMatch::from_match(v, index_map)))
            .collect();
        let named_captures = value
            .named_captures
            .into_iter()
            .map(|(i, v)| (i, OwnedMatch::from_match(v, index_map)))
            .collect();

        Self {
            captures,
            named_captures,
        }
    }
}

#[wasm_bindgen]
impl OwnedCapture {
    pub fn get(&self, index: usize) -> Option<OwnedMatch> {
        self.captures.get(&index).cloned()
    }

    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self, name: &str) -> Option<OwnedMatch> {
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
    use super::{OwnedMatch, RegexEngine};

    #[test]
    fn test_unicode_range() {
        let regex = RegexEngine::new(r#"こ"#);
        let matches = regex.find_all("ここで");

        assert_eq!(
            matches,
            vec![
                OwnedMatch { start: 0, end: 1 },
                OwnedMatch { start: 1, end: 2 },
            ]
        );
    }
}
