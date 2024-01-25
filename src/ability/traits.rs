use std::fmt::Debug;

pub trait Command {
    fn command(&self) -> &'static str;
}

// FIXME this is always a builder, right? ToBuilder? Builder? Buildable?
// FIXME Delegable and make it proven?
pub trait Buildable {
    type Builder: Command + Debug; // FIXME

    fn to_builder(&self) -> Self::Builder;
    fn try_build(builder: Self::Builder) -> Result<Box<Self>, ()>; // FIXME check if this box (for objevt safety) is actually required
}

pub trait Resolvable: Buildable {
    type Awaiting: Command + Debug; // FIXME

    fn to_awaitable(&self) -> Self::Awaiting;
    fn try_into_resolved(promise: Self::Awaiting) -> Result<Box<Self>, ()>; // FIXME check if this box (for objevt safety) is actually required
}

impl<T: Buildable> Command for T {
    fn command(&self) -> &'static str {
        self.to_builder().command()
    }
}

pub trait Runnable {
    type Output;
}

pub trait Ability: Buildable + Runnable {}
// FIXME macro for Delegation (builder) and Promises
