use crate::{did::Did, nonce::Nonce, prove::TryProve};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    cid::{Cid, CidGeneric},
    codec::Encode,
    ipld::Ipld,
    multihash::{Code::Sha2_256, MultihashDigest},
    serde as ipld_serde,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

pub trait Command {
    const COMMAND: &'static str;
}

// FIXME Delegable and make it proven?
pub trait Delegatable: Sized {
    type Builder: Debug + TryInto<Self> + From<Self>;
}

pub trait Resolvable: Delegatable {
    type Awaiting: Debug + TryInto<Self> + From<Self> + Into<Self::Builder>;
}

pub trait Runnable {
    type Output: Debug;
    fn task_id(self, subject: Did, nonce: Nonce) -> Cid;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynJs {
    pub cmd: String,
    pub args: BTreeMap<String, Ipld>,
}

impl Delegatable for DynJs {
    type Builder = Self;
}

impl Resolvable for DynJs {
    type Awaiting = Self;
}

impl Runnable for DynJs {
    type Output = Ipld;

    fn task_id(self, subject: Did, nonce: Nonce) -> Cid {
        let ipld: Ipld = BTreeMap::from_iter([
            ("sub".into(), subject.into()),
            ("do".into(), self.cmd.clone().into()),
            ("args".into(), self.cmd.clone().into()),
            ("nonce".into(), nonce.into()),
        ])
        .into();

        let mut encoded = vec![];
        ipld.encode(DagCborCodec, &mut encoded)
            .expect("should never fail if `encodable_as` is implemented correctly");

        let multihash = Sha2_256.digest(encoded.as_slice());
        CidGeneric::new_v1(DagCborCodec.into(), multihash)
    }
}

impl From<DynJs> for Ipld {
    fn from(js: DynJs) -> Self {
        js.into()
    }
}

impl TryFrom<Ipld> for DynJs {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

////////////////////////
////////////////////////
////////////////////////
////////////////////////
////////////////////////
////////////////////////
////////////////////////
////////////////////////

// trait IntoCheckable {
//     type Checkable;
//     fn to_checkable(&self) -> Self::Checkable;
// }
//
// trait Checkable {
//     type Checker;
//     fn check(me: &Self::Checker, other: &Self::Checker) -> bool;
// }

struct CrudAny;
struct CrudMutate;
struct CrudCreate;
struct CrudUpdate;
struct CrudDestroy;

enum Either<A: ?Sized, B: ?Sized> {
    Left(Box<A>),
    Right(Box<B>),
}

enum Wrapper<'a, T: ?Sized, P> {
    Any,
    Parents(&'a P),
    Me(&'a T),
}

// NOTE must remain unexported!
trait IsChecker {}

impl<T: CheckParents> IsChecker for Parentful<T> {}
impl<T: CheckSelf> IsChecker for Parentless<T> {}

pub trait HasChecker {
    type CheckAs: IsChecker;
}

trait CheckSelf {
    fn check_against_self(&self, other: &Self) -> bool;
}

impl HasChecker for CrudCreate {
    type CheckAs = Parentful<CrudCreate>;
}

impl CheckSelf for CrudCreate {
    fn check_against_self(&self, _other: &Self) -> bool {
        true
    }
}

impl CheckSelf for CrudUpdate {
    fn check_against_self(&self, _other: &Self) -> bool {
        true
    }
}

impl HasChecker for CrudUpdate {
    type CheckAs = Parentful<CrudCreate>;
}

impl CheckSelf for CrudDestroy {
    fn check_against_self(&self, _other: &Self) -> bool {
        true
    }
}

impl HasChecker for CrudDestroy {
    type CheckAs = Parentful<CrudDestroy>;
}

impl CheckSelf for CrudMutate {
    fn check_against_self(&self, _other: &Self) -> bool {
        true
    }
}

impl HasChecker for CrudMutate {
    type CheckAs = Parentful<CrudMutate>;
}

impl CheckSelf for CrudAny {
    fn check_against_self(&self, _other: &Self) -> bool {
        true
    }
}

impl HasChecker for CrudAny {
    type CheckAs = Parentless<CrudAny>;
}

pub enum Parentful<T: CheckParents> {
    Any,
    Parents(T::Parents),
    Me(T),
}

pub enum Parentless<T> {
    Any,
    Me(T),
}

pub trait CheckParents: CheckSelf {
    type Parents;

    fn check_against_parents(&self, other: &Self::Parents) -> bool;
}

pub trait JustCheck<T> {
    fn check<'a>(&'a self, other: &'a T) -> bool;
}

impl<T: CheckSelf> JustCheck<Parentless<T>> for T {
    fn check<'a>(&'a self, other: &'a Parentless<T>) -> bool {
        match other {
            Parentless::Any => true,
            Parentless::Me(me) => self.check_against_self(&me),
        }
    }
}

impl<T: CheckParents> JustCheck<Parentful<T>> for T {
    fn check<'a>(&'a self, other: &'a Parentful<T>) -> bool {
        match other {
            Parentful::Any => true,
            Parentful::Parents(parents) => self.check_against_parents(parents),
            Parentful::Me(me) => self.check_against_self(&me),
        }
    }
}

// trait JustCheckNormalized<T> {}
//
// impl<T> JustCheckNormalized<T> for T {}

// impl<T: CheckParents> JustCheck for T {
//     type ToCheck = Parentful<T, T::Parents>;
// }
//
// trait EvenMoreCheck: JustCheck {
//     fn check<'a>(&'a self, other: Self::ToCheck) -> bool;
// }
//
// impl<T: JustCheck<ToCheck = Parentless<T>>> EvenMoreCheck for T {
//     fn check<'a>(&'a self, other: Parentless<T>) -> bool {
//         match other {
//             Parentless::Any => true,
//             Parentless::Me(me) => self.check_against_self(&me),
//         }
//     }
// }
//
// impl<P, T: JustCheck<ToCheck = Parentful<T, P>>> EvenMoreCheck for T {
//     fn check<'a>(&'a self, other: Parentless<T>) -> bool {
//         match other {
//             Parentful::Any => true,
//             Parentful::Parents(parents) => self.check_against_parents(parents),
//             Parentful::Me(me) => self.check_against_self(&me),
//         }
//     }
// }

// impl<T: CheckParents> JustCheck for T {}

// impl Parentless for CrudAny {}

// TODO note to self, this is effectively a partial order
impl CheckParents for CrudMutate {
    type Parents = CrudAny;

    fn check_against_parents(&self, other: &Self::Parents) -> bool {
        true
    }
}

impl CheckSelf for Either<CrudMutate, CrudAny> {
    fn check_against_self(&self, other: &Self) -> bool {
        match self {
            Either::Left(mutate) => match other {
                Either::Left(other_mutate) => mutate.check_against_self(other_mutate),
                Either::Right(any) => mutate.check_against_parents(any),
            },
            _ => false,
        }
    }
}

impl CheckParents for CrudCreate {
    type Parents = Either<CrudMutate, CrudAny>;

    fn check_against_parents(&self, other: &Self::Parents) -> bool {
        match other {
            Either::Left(mutate) => true,
            Either::Right(any) => true,
        }
    }
}

impl CheckParents for CrudUpdate {
    type Parents = Either<CrudMutate, CrudAny>;

    fn check_against_parents(&self, other: &Self::Parents) -> bool {
        match other {
            Either::Left(mutate) => true,
            Either::Right(any) => true,
        }
    }
}

impl CheckParents for CrudDestroy {
    type Parents = Either<CrudMutate, CrudAny>;

    fn check_against_parents(&self, other: &Self::Parents) -> bool {
        match other {
            Either::Left(mutate) => true,
            Either::Right(any) => true,
        }
    }
}

trait IntoParentsFor<T: CheckParents> {
    fn into_parents(self) -> T::Parents;
}

// impl IntoParentsFor<CrudMutate> for CrudAny {
//     fn into_parents(self) -> Either {
//         todo!()
//     }
// }

enum Void {}

// impl SuperParentalChecker for CrudAny {
//     type SuperParents = Void;
//
//     fn check_against_self(self, other: Self) -> bool {
//         self == other
//     }
//
//     fn check_against_parents(self, other: Self::SuperParents) -> bool {
//         match other {
//             Either::Left(create) => self == create,
//             Either::Right(update) => self == update,
//         }
//     }
// }

enum CrudChecker {
    Create,
    Read,
    Update,
    Delete,
}
