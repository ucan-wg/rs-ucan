use std::fmt::Debug;

// FIXME this is always a builder, right?
pub trait Ability: Sized {
    // FIXME remove sized?
    // pub trait Capability: TryFrom<Ipld> + Into<Ipld> {
    type Builder: From<Self> + TryInto<Self> + PartialEq + Debug; // FIXME

    // fn command(builder: &Self::Builder) -> &'static str;
}

pub trait Builder {
    type Concrete;

    fn command(&self) -> &'static str;
    fn try_build(&self) -> Result<Self::Concrete, ()>; // FIXME
}

// pub trait Builds1 {
//     type B;
// }
//
// impl Builds1 for B::Concrete
// where
//     B: Builder,
// {
//     type B = B;
// }

// FIXME macro for Delegation (builder) and Promises
