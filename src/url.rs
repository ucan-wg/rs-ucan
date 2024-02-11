//! URL utilities.

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
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
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
