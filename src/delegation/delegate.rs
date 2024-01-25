use libipld_core::ipld::Ipld;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum Delegate<T> {
    Any,
    Specific(T),
}

impl<'a, T> From<&'a Delegate<T>> for Ipld
where
    Ipld: From<&'a T>,
{
    fn from(delegate: &'a Delegate<T>) -> Self {
        match delegate {
            Delegate::Any => "ucan/*".into(),
            Delegate::Specific(command) => command.into(),
        }
    }
}

impl<'a, T: TryFrom<&'a Ipld>> TryFrom<&'a Ipld> for Delegate<T> {
    type Error = (); // FIXME

    fn try_from(ipld: &'a Ipld) -> Result<Self, ()> {
        if let ipld_string @ Ipld::String(st) = ipld {
            if st == "ucan/*" {
                Ok(Self::Any)
            } else {
                match T::try_from(ipld_string) {
                    Err(_) => Err(()),
                    Ok(cmd) => Ok(Self::Specific(cmd)),
                }
            }
        } else {
            Err(())
        }
    }
}
