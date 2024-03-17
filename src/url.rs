//! URL utilities.

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use url::Url;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// A wrapper around [`Url`] that has additional trait implementations.
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

impl Newtype {
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        Ok(Newtype(Url::parse(s)?))
    }
}

impl fmt::Display for Newtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Newtype> for Ipld {
    fn from(newtype: Newtype) -> Self {
        Ipld::String(newtype.to_string())
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

#[cfg(feature = "test_utils")]
impl Arbitrary for Newtype {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let url_regex: &str = &r#"[a-zA-Z]+[a-zA-Z0-9]*:(//)?[a-zA-Z0-9._]+(#)?[a-zA-Z0-9_]"#;
        url_regex
            .prop_map(|s| {
                Newtype(Url::parse(&s).expect("the regex generator to create valid URLs"))
            })
            .boxed()
    }
}
