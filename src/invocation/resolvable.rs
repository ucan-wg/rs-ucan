use crate::ability::arguments;
use libipld_core::ipld::Ipld;

pub trait Resolvable: Sized {
    type Promised: Into<arguments::Named<Ipld>>;

    // FIXME indeed needed to get teh right err type
    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised>;

    // FIXME remove
    fn try_resolve0(promised: Self::Promised) -> Result<Self, Self::Promised> {
        Self::try_resolve(promised)
    }
}
