use crate::{ability::traits::Command, prove::TryProve};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Msg {
    to: Url,
    from: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSend {
    to: Url,
    from: Url,
    message: String,
}

// TODO is the to or from often also the subject? Shoudl that be accounted for?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgReceive {
    to: Url,
    from: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    fn from(msg_rcv: MsgReceive) -> Self {
        msg_rcv.into()
    }
}

impl TryFrom<Ipld> for MsgReceiveBuilder {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, ()> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Command for MsgReceive {
    const COMMAND: &'static str = "msg/receive";
}

impl TryFrom<Ipld> for MsgReceive {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl<'a> TryProve<'a, Msg> for Msg {
    type Error = (); // FIXME
    type Proven = Msg;

    fn try_prove(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self == candidate {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl<'a> TryProve<'a, Msg> for MsgSend {
    type Error = (); // FIXME
    type Proven = MsgSend;

    fn try_prove(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self.to == candidate.to && self.from == candidate.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl<'a> TryProve<'a, Msg> for MsgReceive {
    type Error = (); // FIXME
    type Proven = MsgReceive;

    fn try_prove(&'a self, candidate: &'a Msg) -> Result<&'a Self::Proven, ()> {
        if self.to == candidate.to && self.from == candidate.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

// FIXME this needs to work on builders!
impl<'a> TryProve<'a, MsgReceive> for MsgReceive {
    type Error = (); // FIXME
    type Proven = MsgReceive;

    fn try_prove(&'a self, candidate: &'a MsgReceive) -> Result<&'a Self::Proven, ()> {
        if self == candidate {
            Ok(self)
        } else {
            Err(())
        }
    }
}
