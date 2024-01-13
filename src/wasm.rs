use crate::{
    nfa::{StateId, TransitionKind},
    regex::{Capture, Regex},
    Match,
};
use std::collections::HashMap;
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

    #[wasm_bindgen(js_name = "capturesAll")]
    pub fn captures_all(&self, input: &str) -> Vec<RegexCapture> {
        let index_map = get_char_index(input);

        self.engine
            .captures_all(input)
            .into_iter()
            .map(|captures| RegexCapture::from_capture(captures, &index_map))
            .collect()
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
pub struct RegexGroup {
    name: String,
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
impl RegexGroup {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    fn new(name: String, value: Match<'_>, index_map: &HashMap<usize, usize>) -> Self {
        Self {
            name,
            start: index_map[&value.start],
            end: index_map[&value.end],
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RegexCapture {
    groups: Vec<RegexGroup>,
}

#[wasm_bindgen]
impl RegexCapture {
    pub fn groups(&self) -> Vec<RegexGroup> {
        self.groups.clone()
    }

    fn from_capture(value: Capture, index_map: &HashMap<usize, usize>) -> Self {
        let captures = value
            .captures
            .into_iter()
            .map(|(i, v)| RegexGroup::new(i.to_string(), v, index_map));
        let named_captures = value
            .named_captures
            .into_iter()
            .map(|(i, v)| RegexGroup::new(i, v, index_map));

        Self {
            groups: captures.chain(named_captures).collect(),
        }
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

// #[cfg(test)]
// mod tests {
//     use super::{RegexEngine, RegexGroup};
//
//     #[test]
//     fn test_unicode_range() {
//         let regex = RegexEngine::new(r#"ここ"#);
//         let matches = regex.find_all("ここでここで");
//
//         assert_eq!(
//             matches,
//             vec![
//                 RegexGroup { start: 0, end: 2 },
//                 RegexGroup { start: 3, end: 5 },
//             ]
//         );
//
//         let regex = RegexEngine::new(r#"日本語"#);
//         let matches = regex.find_all("これは日本語のテストです。日本語");
//
//         assert_eq!(
//             matches,
//             vec![
//                 RegexGroup { start: 3, end: 6 },
//                 RegexGroup { start: 13, end: 16 },
//             ]
//         );
//     }
// }
