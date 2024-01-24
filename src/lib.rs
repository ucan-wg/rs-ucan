#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! rs-ucan

use std::str::FromStr;

use cid::{multihash, Cid};
use serde::{de, Deserialize, Deserializer, Serialize};

pub mod builder;
pub mod capability;
pub mod crypto;
pub mod did_verifier;
pub mod error;
pub mod plugins;
pub mod semantics;
pub mod store;
pub mod time;
pub mod ucan;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub use linkme;

/// The default multihash algorithm used for UCANs
pub const DEFAULT_MULTIHASH: multihash::Code = multihash::Code::Sha2_256;

/// A decentralized identifier.
pub type Did = String;

use libipld_core::{ipld::Ipld, link::Link};
use std::{collections::BTreeMap, fmt::Debug};
use url::Url;
use void::Void;
use web_time::{Instant, SystemTime};

pub struct InvocationPayload<Ability> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Option<Did>,

    pub ability: Ability, // FIXME check name in spec

    // pub proofs: Vec<Link<Delegation<Ability>>>,
    // pub cause: Option<Link<Receipt<_>>>, // FIXME?
    pub metadata: BTreeMap<Box<str>, Ipld>, // FIXME serde value instead?
    pub nonce: Box<[u8]>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

pub struct DelegationPayload<A: Capability, Cond> {
    pub issuer: Did,
    pub subject: Did,
    pub audience: Did,

    pub capability_builder: A::Builder, // FIXME
    pub conditions: Box<[Cond]>,        // Worth it over a Vec?

    pub metadata: BTreeMap<Box<str>, Ipld>, // FIXME serde value instead?
    pub nonce: Box<[u8]>,                   // Better type?

    pub expiration: Timestamp,
    pub not_before: Option<Timestamp>,
}

// FIXME move that clone?
impl<A: Capability + Clone, Cond> From<InvocationPayload<A>> for DelegationPayload<A, Cond> {
    fn from(invocation: InvocationPayload<A>) -> Self {
        Self {
            issuer: invocation.issuer.clone(),
            subject: invocation.subject.clone(),
            audience: invocation
                .audience
                .clone()
                .unwrap_or(invocation.issuer.clone()),
            capability_builder: <A as Into<A::Builder>>::into(invocation.ability.clone()),
            conditions: Box::new([]),
            metadata: invocation.metadata.clone(),
            nonce: invocation.nonce.clone(),
            expiration: invocation.expiration.clone(),
            not_before: invocation.not_before.clone(),
        }
    }
}

// impl<A: Capability, Cond> DelegationPayload<A, Cond> {
//     fn check(
//         &self,
//         proof: &DelegationPayload<impl TryProven<A> + Capability, Cond>,
//         now: SystemTime,
//     ) -> Result<(), ()> {
//         // FIXME heavily WIP
//         // FIXME signature
//         self.check_time(now).unwrap();
//         self.check_issuer(&proof.audience)?; // FIXME alignment
//         self.check_subject(&proof.subject)?;
//         self.check_conditions(&proof.conditions)?;
//
//         proof.check_expiration(now)?;
//         proof.check_not_before(now)?;
//
//         self.check_ability(&proof.capability_builder)?;
//         Ok(())
//     }
// }

// FIXME add is_roo t

pub trait TryProven<A> {
    type Proven1;
    type Error1;
    fn try_proven<'a>(&'a self, candidate: &'a A) -> Result<&'a Self::Proven1, Self::Error1>;
}

impl<T, U> TryProven<T> for U
where
    T: TryProve<U>,
{
    type Proven1 = T::Proven;
    type Error1 = T::Error;

    fn try_proven<'a>(&'a self, candidate: &'a T) -> Result<&'a T::Proven, T::Error> {
        candidate.try_prove(self)
    }
}

pub struct DelegateAny;

// FIXME ToBuilder

// FIXME Remove
// pub trait Prove<T> {
//     type Witness;
//     // FIXME make sure that passing the top-level item through and not checking each
//     // item in the chain against the next one is correct in the 1.0 semantics
//     fn prove<'a>(&'a self, proof: &'a T) -> &Self::Witness;
// }

impl<T> TryProve<DelegateAny> for T {
    type Error = Void;
    type Proven = T;

    fn try_prove<'a>(&'a self, _proof: &'a DelegateAny) -> Result<&'a Self::Proven, Void> {
        Ok(self)
    }
}

// impl<T> Prove<DelegateAny> for T {
//     type Witness = T;
//
//     fn prove<'a>(&'a self, proof: &'a DelegateAny) -> &Self::Witness {
//         self
//     }
// }
//
// impl<T: Prove<T>> TryProve<T> for T {
//     type Error = Void;
//     type Proven = T;
//
//     fn try_prove<'a>(&'a self, proof: &'a T) -> Result<&'a T, Void> {
//         Ok(proof)
//     }
// }

#[cfg_attr(doc, aquamarine::aquamarine)]
/// FIXME
///
/// ```mermaid
/// flowchart LR
///   Invocation --> more --> Self --> Candidate --> more2
///   more[...]
///   more2[...]
/// ```
pub trait TryProve<T> {
    type Error;
    type Proven;

    fn try_prove<'a>(&'a self, candidate: &'a T) -> Result<&'a Self::Proven, Self::Error>;
}

/////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg {
    to: Url,
    from: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgSend {
    to: Url,
    from: Url,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgReceive {
    to: Url,
    from: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgReceiveBuilder {
    to: Option<Url>,
    from: Option<Url>,
}

impl From<MsgReceive> for MsgReceiveBuilder {
    fn from(msg: MsgReceive) -> Self {
        Self {
            to: Some(msg.to),
            from: Some(msg.from),
        }
    }
}

impl TryFrom<MsgReceiveBuilder> for MsgReceive {
    type Error = MsgReceiveBuilder;

    fn try_from(builder: MsgReceiveBuilder) -> Result<Self, MsgReceiveBuilder> {
        // FIXME
        if let (Some(to), Some(from)) = (builder.clone().to, builder.clone().from) {
            Ok(Self { to, from })
        } else {
            Err(builder.clone()) // FIXME
        }
    }
}

impl From<MsgReceive> for Ipld {
    fn from(msg: MsgReceive) -> Self {
        let mut map = BTreeMap::new();
        map.insert("to".into(), msg.to.to_string().into());
        map.insert("from".into(), msg.from.to_string().into());
        map.into()
    }
}

impl TryFrom<&Ipld> for MsgReceiveBuilder {
    type Error = ();

    fn try_from(ipld: &Ipld) -> Result<Self, ()> {
        match ipld {
            Ipld::Map(map) => {
                if map.len() > 2 {
                    return Err(()); // FIXME
                }

                // FIXME
                let to = if let Some(Ipld::String(to)) = map.get("to") {
                    Url::from_str(to).ok() // FIXME
                } else {
                    None
                };

                let from = if let Some(Ipld::String(from)) = map.get("to") {
                    Url::from_str(from).ok() // FIXME
                } else {
                    None
                };

                Ok(Self { to, from })
            }
            _ => Err(()),
        }
    }
}

impl Capability for MsgReceive {
    type Builder = MsgReceiveBuilder;
    const COMMAND: &'static str = "msg/receive";
}

impl TryFrom<&Ipld> for MsgReceive {
    type Error = (); // FIXME

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryProve<Msg> for Msg {
    type Error = (); // FIXME
    type Proven = Msg;

    fn try_prove<'a>(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self == candidate {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl TryProve<Msg> for MsgSend {
    type Error = (); // FIXME
    type Proven = MsgSend;

    fn try_prove<'a>(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self.to == candidate.to && self.from == candidate.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl TryProve<Msg> for MsgReceive {
    type Error = (); // FIXME
    type Proven = MsgReceive;

    fn try_prove<'a>(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self.to == candidate.to && self.from == candidate.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

// FIXME this needs to work on builders!
impl TryProve<MsgReceive> for MsgReceive {
    type Error = (); // FIXME
    type Proven = MsgReceive;

    fn try_prove<'a>(&'a self, candidate: &'a MsgReceive) -> Result<&'a Self::Proven, ()> {
        if self == candidate {
            Ok(self)
        } else {
            Err(())
        }
    }
}

///////////

// FIXME remove
// impl<T, P: Prove<T>> TryProve<T> for P {
//     type Error = Void;
//     type Proven = P::Witness;
//
//     fn try_prove<'a>(&'a self, candidate: &'a T) -> Result<&'a Self::Proven, Void> {
//         Ok(self.prove(candidate))
//     }
// }

impl TryProve<CrudDestroy> for CrudDestroy {
    type Error = (); // FIXME
    type Proven = CrudDestroy;
    fn try_prove<'a>(&'a self, candidate: &'a CrudDestroy) -> Result<&'a Self::Proven, ()> {
        if self.uri == candidate.uri {
            Ok(self)
        } else {
            Err(())
        }
    }
}

// FIXME ProveWith<Crud>?
impl TryProve<CrudMutate> for CrudDestroy {
    type Error = (); // FIXME
    type Proven = CrudDestroy;

    fn try_prove<'a>(&'a self, candidate: &'a CrudMutate) -> Result<&'a Self::Proven, ()> {
        if self.uri == candidate.uri {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl TryProve<CrudRead> for CrudRead {
    type Error = ();
    type Proven = CrudRead;

    fn try_prove<'a>(&'a self, candidate: &'a CrudRead) -> Result<&'a Self::Proven, ()> {
        if self.uri == candidate.uri {
            // FIXME contains & args
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl TryProve<Crud> for CrudRead {
    type Error = (); // FIXME
    type Proven = CrudRead;

    fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a Self::Proven, ()> {
        if self.uri == candidate.uri {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl TryProve<Crud> for CrudMutate {
    type Error = (); // FIXME
    type Proven = CrudMutate;

    fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a Self::Proven, ()> {
        if self.uri == candidate.uri {
            Ok(self)
        } else {
            Err(())
        }
    }
}

// FIXME
impl<C: TryProve<CrudMutate, Proven = C>> TryProve<Crud> for C {
    type Error = ();
    type Proven = C;

    // FIXME
    fn try_prove<'a>(&'a self, candidate: &'a Crud) -> Result<&'a C, ()> {
        match self.try_prove(&CrudMutate {
            uri: candidate.uri.clone(),
        }) {
            Ok(_) => Ok(self),
            Err(_) => Err(()),
        }
    }
}

// FIXME lives etirely in bindgen
// https://rustwasm.github.io/docs/wasm-bindgen/contributing/design/importing-js-struct.html
// pub struct DynamicJs {
//     pub command: Box<str>,
//     pub args: BTreeMap<Box<str>, Ipld>,
// }
//
// impl TryProve<DynamicJs> for DynamicJs {
//     type Error = ();
//     type Proven = DynamicJs;
//
//     fn try_prove<'a>(&'a self, candidate: &'a DynamicJs) -> Result<&'a DynamicJs, ()> {
//
//     }
// }

// impl ProveDelegaton<DynamicJs> for DynamicJs {
//     type Error = anyhow::Error;
//
//     fn prove(&self, proof: &DynamicJs) -> Result<Self, anyhow::Error> {
//         todo!()
//     }
// }
//

pub struct Crud {
    uri: Url,
}

pub struct CrudMutate {
    uri: Url,
}

pub struct CrudCreate {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>,
}

pub struct CrudRead {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>, // FIXME need these?
}

pub struct CrudUpdate {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>,
}

pub struct CrudDestroy {
    pub uri: Url,
}

// impl Capabilty for CrudRead{
//     const COMMAND = "crud/read";
//
//     fn subject(&self) -> Did {
//         todo!()
//     }
// }
//
// pub enum Condition {
//     Contains { field: &str, value: Vec<Ipld> },
//     MinLength { field: &str, value: u64 },
//     MaxLength { field: &str, value: u64 },
//     Equals { field: &str, value: Ipld },
//     Regex { field: &str }, // FIXME
//
//     // Silly example
//     OnDayOfWeek { day: Day },
// }

pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

// pub trait CapBuilder: Default {
//     type Ability;
//     fn build(self) -> Result<Capability, Error>;
// }

// pub trait BuilderFor<Ability>: CapBuilder {
//     type Builder: CapBuilder<Ability>;
// }

pub trait Capability: Sized {
    // pub trait Capability: Into<Ipld> {
    // FIXME remove sized?
    // pub trait Capability: TryFrom<Ipld> + Into<Ipld> {
    type Builder: From<Self> + TryInto<Self> + PartialEq + Debug;
    const COMMAND: &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsTime {
    time: SystemTime,
}

// FIXME just lifting this from Elixir for now
pub struct OutOfRangeError {
    pub tried: SystemTime,
}

impl JsTime {
    // FIXME value should be a system time?
    pub fn new(time: SystemTime) -> Result<Self, OutOfRangeError> {
        if time
            .duration_since(std::time::UNIX_EPOCH)
            .expect("FIXME")
            .as_secs()
            > 0x1FFFFFFFFFFFFF
        {
            Err(OutOfRangeError { tried: time })
        } else {
            Ok(JsTime { time })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Timestamp {
    Sending(JsTime),
    Receiving(SystemTime),
}

// pub struct ReceiptPayload<T> {
//     pub issuer: Did,
//     pub ran: Link<Invocation<T>>,
//     pub out: UcanResult<T>, // FIXME?
//     pub proofs: Vec<Link<Delegation<FIXME>>>,
//     pub metadata: BTreeMap<str, Ipld>, // FIXME serde value instead?
//     pub issued_at: u64,
// }
//
// pub enum UcanResult<T> {
//     UcanOk(T),
//     UcanErr(BTreeMap<&str, Ipld>),
// }
//
// pub struct Capability<T> {
//     command: String,
//     payload: BTreeMap<str, T>,
// }

//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////

/// The empty fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyFact {}

/// The default fact
pub type DefaultFact = EmptyFact;

/// A newtype around Cid that (de)serializes as a string
#[derive(Debug, Clone)]
pub struct CidString(pub(crate) Cid);

impl Serialize for CidString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for CidString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Cid::from_str(&s)
            .map(CidString)
            .map_err(|e| de::Error::custom(format!("invalid CID: {}", e)))
    }
}

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
