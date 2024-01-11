use crate::{
    nfa::{StateId, TransitionKind},
    regex::{Capture, Match, Regex},
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
        self.engine.captures(input).map(OwnedCapture::from)
    }

    pub fn find(&self, input: &str) -> Option<OwnedMatch> {
        self.engine.find(input).map(OwnedMatch::from)
    }

    #[wasm_bindgen(js_name = "findAll")]
    pub fn find_all(&self, input: &str) -> Vec<OwnedMatch> {
        self.engine
            .find_all(input)
            .into_iter()
            .map(OwnedMatch::from)
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
#[derive(Clone)]
pub struct OwnedMatch {
    pub start: usize,
    pub end: usize,
}

impl From<Match<'_>> for OwnedMatch {
    fn from(value: Match<'_>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[wasm_bindgen]
pub struct OwnedCapture {
    captures: BTreeMap<usize, OwnedMatch>,
    named_captures: HashMap<String, OwnedMatch>,
}

impl From<Capture<'_>> for OwnedCapture {
    fn from(value: Capture) -> Self {
        let captures = value
            .captures
            .into_iter()
            .map(|(i, v)| (i, OwnedMatch::from(v)))
            .collect();
        let named_captures = value
            .named_captures
            .into_iter()
            .map(|(i, v)| (i, OwnedMatch::from(v)))
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
