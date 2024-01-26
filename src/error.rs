//! Error types for UCAN

use thiserror::Error;

/// Error types for UCAN
#[derive(Error, Debug)]
pub enum Error {
    /// Parsing errors
    #[error("An error occurred while parsing the token: {msg}")]
    TokenParseError {
        /// Error message
        msg: String,
    },
    /// Verification errors
    #[error("An error occurred while verifying the token: {msg}")]
    VerifyingError {
        /// Error message
        msg: String,
    },
    /// Signing errors
    #[error("An error occurred while signing the token: {msg}")]
    SigningError {
        /// Error message
        msg: String,
    },
    /// Plugin errors
    #[error(transparent)]
    PluginError(PluginError),
    /// Internal errors
    #[error("An unexpected error occurred in ucan: {msg}\n\nThis is a bug: please consider filing an issue at https://github.com/ucan-wg/ucan/issues")]
    InternalUcanError {
        /// Error message
        msg: String,
    },
}

/// Error types for plugins
#[derive(Error, Debug)]
#[error(transparent)]
pub struct PluginError {
    #[from]
    inner: anyhow::Error,
}

impl From<anyhow::Error> for Error {
    fn from(inner: anyhow::Error) -> Self {
        Self::PluginError(PluginError { inner })
    }
}
