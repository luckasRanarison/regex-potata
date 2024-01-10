use crate::{
    error::Error,
    nfa::TransitionMap,
    regex::{Capture, Match, Regex},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RegexEngine {
    engine: Regex,
}

impl<'a> RegexEngine {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        Ok(Self {
            engine: Regex::new(pattern)?,
        })
    }

    pub fn captures(&self, input: &'a str) -> Option<Capture<'a>> {
        self.engine.captures(input)
    }

    pub fn find(&self, input: &'a str) -> Option<Match<'a>> {
        self.engine.find(input)
    }

    pub fn find_all(&self, input: &'a str) -> Vec<Match<'a>> {
        self.engine.find_all(input)
    }

    pub fn test(&self, input: &'a str) -> bool {
        self.engine.test(input)
    }

    pub fn transitions(&self) -> &TransitionMap {
        self.engine.transitions()
    }
}
