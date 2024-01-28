use crate::{
    ability::traits::{CheckParents, CheckSelf, HasChecker},
    promise::Promise,
    prove::TryProve,
};
use std::{collections::BTreeMap, fmt::Debug};
use url::Url;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld

pub struct Crud {
    pub uri: Url,
}

pub struct CrudRead {
    pub uri: Url,
}

pub struct CrudMutate {
    pub uri: Url,
}

pub struct CrudCreate {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrudUpdate {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrudDestroy {
    pub uri: Url,
}

pub struct CrudAny;
pub struct CrudMutate;
pub struct CrudCreate;
pub struct CrudUpdate;
pub struct CrudDestroy;
pub struct CrudRead;

pub enum CrudParents {
    MutableParent(CrudMutate),
    AnyParent(CrudAny),
}

impl HasChecker for CrudCreate {
    type CheckAs = Parentful<CrudCreate>;
}

impl CheckSelf for CrudCreate {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl CheckSelf for CrudUpdate {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl HasChecker for CrudUpdate {
    type CheckAs = Parentful<CrudCreate>;
}

impl CheckSelf for CrudDestroy {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl HasChecker for CrudDestroy {
    type CheckAs = Parentful<CrudDestroy>;
}

impl CheckSelf for CrudMutate {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl HasChecker for CrudMutate {
    type CheckAs = Parentful<CrudMutate>;
}

impl CheckSelf for CrudAny {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl HasChecker for CrudAny {
    type CheckAs = Parentless<CrudAny>;
}

// TODO note to self, this is effectively a partial order
impl CheckParents for CrudMutate {
    type Parents = CrudAny;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}

impl CheckSelf for CrudParents {
    type SelfError = ();
    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        match self {
            CrudParents::MutableParent(mutate) => match other {
                CrudParents::MutableParent(other_mutate) => mutate.check_against_self(other_mutate),
                CrudParents::AnyParent(any) => mutate.check_against_parents(any),
            },
            _ => Err(()),
        }
    }
}

impl CheckParents for CrudCreate {
    type Parents = CrudParents;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            CrudParents::MutableParent(mutate) => Ok(()),
            CrudParents::AnyParent(any) => Ok(()),
        }
    }
}

impl CheckParents for CrudUpdate {
    type Parents = CrudParents;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            CrudParents::MutableParent(mutate) => Ok(()),
            CrudParents::AnyParent(any) => Ok(()),
        }
    }
}

impl CheckParents for CrudDestroy {
    type Parents = CrudParents;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            CrudParents::MutableParent(mutate) => Ok(()),
            CrudParents::AnyParent(any) => Ok(()),
        }
    }
}

impl CheckSelf for CrudRead {
    type SelfError = ();
    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl CheckParents for CrudRead {
    type Parents = CrudAny;
    type ParentError = ();
    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}

// FIXME these should probably be behind a feature flag

// impl Capabilty for CrudRead{
//     const COMMAND = "crud/read";
//
//     fn subject(&self) -> Did {
//         todo!()
//     }
// }

// impl TryProve<CrudDestroy> for CrudDestroy {
//     type Error = (); // FIXME
//     type Proven = CrudDestroy;
//     fn try_prove<'a>(&'a self, proof: &'a CrudDestroy) -> Result<&'a Self::Proven, ()> {
//         if self.uri == proof.uri {
//             Ok(self)
//         } else {
//             Err(())
//         }
//     }
// }
//
// // FIXME ProveWith<Crud>?
// impl TryProve<CrudMutate> for CrudDestroy {
//     type Error = (); // FIXME
//     type Proven = CrudDestroy;
//
//     fn try_prove<'a>(&'a self, proof: &'a CrudMutate) -> Result<&'a Self::Proven, ()> {
//         if self.uri == proof.uri {
//             Ok(self)
//         } else {
//             Err(())
//         }
//     }
// }
//
// impl TryProve<CrudRead> for CrudRead {
//     type Error = ();
//     type Proven = CrudRead;
//
//     fn try_prove<'a>(&'a self, proof: &'a CrudRead) -> Result<&'a Self::Proven, ()> {
//         if self.uri == proof.uri {
//             // FIXME contains & args
//             Ok(self)
//         } else {
//             Err(())
//         }
//     }
// }
//
// impl TryProve<Crud> for CrudRead {
//     type Error = (); // FIXME
//     type Proven = CrudRead;
//
//     fn try_prove<'a>(&'a self, proof: &'a Crud) -> Result<&'a Self::Proven, ()> {
//         if self.uri == proof.uri {
//             Ok(self)
//         } else {
//             Err(())
//         }
//     }
// }
//
// impl TryProve<Crud> for CrudMutate {
//     type Error = (); // FIXME
//     type Proven = CrudMutate;
//
//     fn try_prove<'a>(&'a self, proof: &'a Crud) -> Result<&'a Self::Proven, ()> {
//         if self.uri == proof.uri {
//             Ok(self)
//         } else {
//             Err(())
//         }
//     }
// }
//
// // FIXME
// impl<C: TryProve<CrudMutate, Proven = C>> TryProve<Crud> for C {
//     type Error = ();
//     type Proven = C;
//
//     // FIXME
//     fn try_prove<'a>(&'a self, proof: &'a Crud) -> Result<&'a C, ()> {
//         match self.try_prove(&CrudMutate {
//             uri: proof.uri.clone(),
//         }) {
//             Ok(_) => Ok(self),
//             Err(_) => Err(()),
//         }
//     }
// }
