use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum ParseError {
    #[error("unmatched trailing input")]
    TrailingInput(String),

    #[error("unknown pattern: {0}")]
    UnknownPattern(String),

    #[error("missing starting dot: {0}")]
    MissingStartingDot(String),

    #[error("starts with double dot: {0}")]
    StartsWithDoubleDot(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum SelectorErrorReason {
    #[error("Index out of bounds")]
    IndexOutOfBounds,

    #[error("Key not found")]
    KeyNotFound,

    #[error("Not a list")]
    NotAList,

    #[error("Not a map")]
    NotAMap,

    #[error("Not a collection")]
    NotACollection,

    #[error("Not a number")]
    NotANumber,

    #[error("Not a string")]
    NotAString,
}
