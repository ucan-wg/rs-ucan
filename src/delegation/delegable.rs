use crate::{
    ability::{
        arguments,
        command::ToCommand,
        parse::{ParseAbility, ParseAbilityError},
    },
    proof::checkable::Checkable,
};
use libipld_core::ipld::Ipld;

/// A trait for types that can be delegated.
///
/// Since [`Delegation`]s may omit fields (until [`Invocation`]),
/// this trait helps associate the delegatable variant to the invocable one.
///
/// [`Delegation`]: crate::delegation::Delegation
/// [`Invocation`]: crate::invocation::Invocation
// FIXME NOTE: don't need parse ability, because parse -> builder -> self
// FIXME NOTE: don't need ToCommand ability, because parse -> builder -> self, or .. -> promieed -> ..
pub trait Delegable: Sized {
    /// A delegation with some arguments filled.
    type Builder: TryInto<Self>
        + From<Self>
        + Checkable
        + ParseAbility
        + ToCommand
        + Into<arguments::Named<Ipld>>;

    fn into_command(self) -> String {
        Self::Builder::from(self).to_command()
    }

    fn into_named_args(self) -> arguments::Named<Ipld> {
        Self::Builder::from(self).into()
    }

    fn try_parse_to_ready(
        command: &str,
        named: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<<Self::Builder as ParseAbility>::ArgsErr>> {
        let builder = Self::Builder::try_parse(command, named)?;
        builder.try_into().map_err(|err| todo!())
    }

    fn try_from_named(
        named: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<<Self::Builder as ParseAbility>::ArgsErr>> {
        let builder = Self::Builder::try_parse("", named)?;
        builder.try_into().map_err(|err| todo!())
    }
}
