use crate::ipld;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collection {
    Array(Vec<ipld::Newtype>),
    Map(BTreeMap<String, ipld::Newtype>),
}

impl From<Collection> for Ipld {
    fn from(collection: Collection) -> Self {
        match collection {
            Collection::Array(xs) => Ipld::List(xs.into_iter().map(Into::into).collect()),
            Collection::Map(xs) => Ipld::Map(
                xs.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<BTreeMap<String, Ipld>>(),
            ),
        }
    }
}

impl Collection {
    pub fn to_vec(self) -> Vec<ipld::Newtype> {
        match self {
            Collection::Array(xs) => xs,
            Collection::Map(xs) => xs.into_values().collect(),
        }
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Collection {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            prop::collection::vec(ipld::Newtype::arbitrary(), 0..10).prop_map(Collection::Array),
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..10)
                .prop_map(Collection::Map),
        ]
        .boxed()
    }
}
