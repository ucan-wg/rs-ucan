//! URL utilities.

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// A wrapper around [`Url`] that has additional trait implementations
///
/// Usage is very simple: wrap a [`Newtype`] to gain access to additional traits and methods.
///
/// ```rust
/// # use ::url::Url;
/// # use ucan::url;
/// #
/// let url = Url::parse("https://example.com").unwrap();
/// let wrapped = url::Newtype(url.clone());
/// // wrapped.some_trait_method();
/// ```
///
/// Unwrap a [`Newtype`] to use any interfaces that expect plain [`Ipld`].
///
/// ```
/// # use ::url::Url;
/// # use ucan::url;
/// #
/// # let url = Url::parse("https://example.com").unwrap();
/// # let wrapped = url::Newtype(url.clone());
/// #
/// assert_eq!(wrapped.0, url);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Newtype(pub Url);

impl From<Newtype> for Ipld {
    fn from(newtype: Newtype) -> Self {
        newtype.into()
    }
}

impl TryFrom<Ipld> for Newtype {
    type Error = FromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => Url::parse(&s)
                .map(Newtype)
                .map_err(FromIpldError::UrlParseError),
            _ => Err(FromIpldError::NotAString),
        }
    }
}

/// Possible errors when trying to convert from [`Ipld`].
#[derive(Debug, Error)]
pub enum FromIpldError {
    /// Not an IPLD string.
    #[error("Not an IPLD string")]
    NotAString,

    /// Failed to parse the URL.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}
