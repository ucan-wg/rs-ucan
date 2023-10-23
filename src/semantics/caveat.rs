//! UCAN Caveats

use std::fmt;

use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};
use erased_serde::serialize_trait_object;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

/// A caveat defined as part of a semantics
pub trait Caveat: Send + Sync + DynClone + Downcast + erased_serde::Serialize + 'static {
    /// Returns true if the caveat is valid
    fn is_valid(&self) -> bool;

    /// Returns true if self is a valid attenuation of other
    fn is_valid_attenuation(&self, other: &dyn Caveat) -> bool;
}

clone_trait_object!(Caveat);
impl_downcast!(Caveat);
serialize_trait_object!(Caveat);

impl fmt::Debug for dyn Caveat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Caveat({})", std::any::type_name::<Self>())
    }
}

/// A caveat that is always valid
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EmptyCaveat;

impl Caveat for EmptyCaveat {
    fn is_valid(&self) -> bool {
        true
    }

    fn is_valid_attenuation(&self, other: &dyn Caveat) -> bool {
        if let Some(resource) = other.downcast_ref::<Self>() {
            return self == resource;
        };

        false
    }
}

impl Caveat for Box<dyn Caveat> {
    fn is_valid(&self) -> bool {
        (**self).is_valid()
    }

    fn is_valid_attenuation(&self, other: &dyn Caveat) -> bool {
        (**self).is_valid_attenuation(other)
    }
}

impl<'de> Deserialize<'de> for EmptyCaveat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NoFieldsVisitor;

        impl<'de> Visitor<'de> for NoFieldsVisitor {
            type Value = EmptyCaveat;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an empty object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if let Some(field) = map.next_key()? {
                    return Err(serde::de::Error::unknown_field(field, &[]));
                }

                Ok(EmptyCaveat)
            }
        }

        deserializer.deserialize_map(NoFieldsVisitor)
    }
}

impl Serialize for EmptyCaveat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}
