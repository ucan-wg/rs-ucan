//! UCAN Facts

use std::{any::Any, fmt};

use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};
use serde::{Deserialize, Serialize};

/// A fact defined as part of a semantics
pub trait Fact: DynClone + Downcast + 'static {}

clone_trait_object!(Fact);
impl_downcast!(Fact);

impl<T> Fact for T where T: Any + Clone {}

impl fmt::Debug for dyn Fact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fact({})", std::any::type_name::<Self>())
    }
}

/// The empty fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultFact {}
