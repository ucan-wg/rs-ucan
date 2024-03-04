//! Helpers for working with [`Ipld`] numerics.

use enum_as_inner::EnumAsInner;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// The union of [`Ipld`] numeric types
///
/// This is helpful when comparing different numeric types, such as
/// bounds checking in [`Predicate`]s.
///
/// [`Predicate`]: crate::delegation::policy::predicate::Predicate
#[derive(Debug, Clone, PartialEq, EnumAsInner, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    /// Designate a floating point number
    Float(f64),

    /// Designate an integer
    Integer(i128),
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Number::Float(a), Number::Float(b)) => a.partial_cmp(b),
            (Number::Integer(a), Number::Integer(b)) => a.partial_cmp(b),
            (Number::Float(a), Number::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Number::Integer(a), Number::Float(b)) => (*a as f64).partial_cmp(b),
        }
    }
}

impl From<Number> for Ipld {
    fn from(number: Number) -> Self {
        number.into()
    }
}

impl TryFrom<Ipld> for Number {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<i128> for Number {
    fn from(i: i128) -> Number {
        Number::Integer(i)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Number {
        Number::Float(f)
    }
}

impl Arbitrary for Number {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<f64>().prop_map(Number::Float),
            any::<i128>().prop_map(Number::Integer),
        ]
        .boxed()
    }
}
