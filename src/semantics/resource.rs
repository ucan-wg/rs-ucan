//! UCAN Resources

use std::fmt::{self, Display};

use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};

/// A resource defined as part of a semantics
pub trait Resource: Send + Sync + Display + DynClone + Downcast + 'static {
    /// Returns true if self is a valid attenuation of other
    fn is_valid_attenuation(&self, other: &dyn Resource) -> bool;
}

clone_trait_object!(Resource);
impl_downcast!(Resource);

impl Resource for Box<dyn Resource> {
    fn is_valid_attenuation(&self, other: &dyn Resource) -> bool {
        (**self).is_valid_attenuation(other)
    }
}

impl fmt::Debug for dyn Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"Resource("{}")"#, self)
    }
}
