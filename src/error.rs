use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    ParsingError(ParsingError),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Missing {0}")]
    MissingCharacter(char),
    #[error("Invalid escape sequence")]
    InvalidEscapeSequence,
    #[error("Invalid quantifier")]
    InvalidQuantifier,
}
