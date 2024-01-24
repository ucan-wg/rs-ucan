use libipld_core::link::Link;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum Promise<T>
where
    T: Debug + Clone + PartialEq,
{
    PromiseOk(Link<T>),
    PromiseErr(Link<T>),
    PromiseAny(Link<T>),
}
