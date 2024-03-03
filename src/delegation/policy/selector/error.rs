use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum ParseError {
    #[error("unmatched trailing input")]
    TrailingInput(String),

    #[error("unknown pattern: {0}")]
    UnknownPattern(String),
}
