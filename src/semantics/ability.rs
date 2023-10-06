//! UCAN Abilities

use std::fmt::{self, Display};

use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};

use super::caveat::Caveat;

/// An ability defined as part of a semantics
pub trait Ability: Send + Sync + Display + DynClone + Downcast + 'static {
    /// Returns true if self is a valid attenuation of other
    fn is_valid_attenuation(&self, other: &dyn Ability) -> bool;

    /// Returns true if caveat is a valid caveat for self
    fn is_valid_caveat(&self, _caveat: &dyn Caveat) -> bool {
        false
    }
}

clone_trait_object!(Ability);
impl_downcast!(Ability);

impl Ability for Box<dyn Ability> {
    fn is_valid_attenuation(&self, other: &dyn Ability) -> bool {
        (**self).is_valid_attenuation(other)
    }
}

impl fmt::Debug for dyn Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"Ability("{}")"#, self)
    }
}

/// The top ability
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopAbility;

impl Ability for TopAbility {
    fn is_valid_attenuation(&self, other: &dyn Ability) -> bool {
        if let Some(ability) = other.downcast_ref::<Self>() {
            return self == ability;
        };

        false
    }
}

impl Display for TopAbility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "*")
    }
}
