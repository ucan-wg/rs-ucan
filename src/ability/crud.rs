use crate::{promise::Promise, prove::TryProve};
use std::{collections::BTreeMap, fmt::Debug};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum Field<T>
where
    T: Debug + Clone + PartialEq,
{
    Value(T),
    Await(Promise), // FIXME
}

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld

pub struct Crud {
    uri: Field<Url>,
}

pub struct CrudRead {
    pub uri: Field<Url>,
}

pub struct CrudMutate {
    uri: Field<Url>,
}

pub struct CrudCreate {
    pub uri: Field<Url>,
    pub args: BTreeMap<Box<str>, Field<String>>,
}

pub struct CrudUpdate {
    pub uri: Field<Url>,
    pub args: BTreeMap<Box<str>, Field<String>>,
}

pub struct CrudDestroy {
    pub uri: Field<Url>,
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
//     fn try_prove<'a>(&'a self, candidate: &'a CrudDestroy) -> Result<&'a Self::Proven, ()> {
//         if self.uri == candidate.uri {
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
//     fn try_prove<'a>(&'a self, candidate: &'a CrudMutate) -> Result<&'a Self::Proven, ()> {
//         if self.uri == candidate.uri {
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
//     fn try_prove<'a>(&'a self, candidate: &'a CrudRead) -> Result<&'a Self::Proven, ()> {
//         if self.uri == candidate.uri {
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
//     fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a Self::Proven, ()> {
//         if self.uri == candidate.uri {
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
//     fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a Self::Proven, ()> {
//         if self.uri == candidate.uri {
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
//     fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a C, ()> {
//         match self.try_prove(&CrudMutate {
//             uri: candidate.uri.clone(),
//         }) {
//             Ok(_) => Ok(self),
//             Err(_) => Err(()),
//         }
//     }
// }
