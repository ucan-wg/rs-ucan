use std::fmt::Debug;

pub trait Ability: Sized {
    // FIXME remove sized?
    // pub trait Capability: TryFrom<Ipld> + Into<Ipld> {
    type Builder: From<Self> + TryInto<Self> + PartialEq + Debug; // FIXME
    const COMMAND: &'static str;
}

// FIXME macro for Delegation (builder) and Promises
