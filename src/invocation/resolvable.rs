use crate::{ability::arguments, delegation::Delegable};
use libipld_core::ipld::Ipld;

pub trait Resolvable: Delegable {
    // FIXME rename "Unresolved"
    type Promised: Into<Self::Builder> + Into<arguments::Named<Ipld>>;

    // FIXME indeed needed to get teh right err type
    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised>;

    // FIXME better name
    // NOTE this takes anything taht doesn't resolve and returns None on those fields
    // FIXME no, jsut use Into<Builder> and NOTE THIS IN THE DOCS
    // fn resolve_to_builder(&self) -> Self::Builder;
}

// impl Delegable for Ipld {
//     type Builder = Option<Ipld>;
// }
