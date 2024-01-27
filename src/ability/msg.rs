use crate::{
    ability::traits::{Command, Delegatable, Resolvable},
    promise::Deferrable,
    prove::TryProve,
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Msg {
    to: Url,
    from: Url,
}

impl Command for Msg {
    const COMMAND: &'static str = "msg/*";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<Url>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgDeferrable {
    to: Deferrable<Url>,
    from: Deferrable<Url>,
}

impl Delegatable for Msg {
    type Builder = MsgBuilder;
}

impl Resolvable for Msg {
    type Awaiting = MsgDeferrable;
}

impl From<Msg> for MsgBuilder {
    fn from(msg: Msg) -> Self {
        Self {
            to: Some(msg.to),
            from: Some(msg.from),
        }
    }
}

impl TryFrom<MsgBuilder> for Msg {
    type Error = ();

    fn try_from(builder: MsgBuilder) -> Result<Self, Self::Error> {
        if let (Some(to), Some(from)) = (builder.clone().to, builder.clone().from) {
            Ok(Self { to, from })
        } else {
            Err(()) // FIXME
        }
    }
}

impl From<Msg> for MsgDeferrable {
    fn from(msg: Msg) -> Self {
        Self {
            to: Deferrable::Resolved(msg.to),
            from: Deferrable::Resolved(msg.from),
        }
    }
}

impl TryFrom<MsgDeferrable> for Msg {
    type Error = ();

    fn try_from(deferable: MsgDeferrable) -> Result<Self, Self::Error> {
        if let (Deferrable::Resolved(to), Deferrable::Resolved(from)) =
            (deferable.to, deferable.from)
        {
            Ok(Self { to, from })
        } else {
            Err(()) // FIXME
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSend {
    to: Url,
    from: Url,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendDeferrable {
    to: Deferrable<Url>,
    from: Deferrable<Url>,
    message: Deferrable<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgReceiveDeferrable {
    to: Deferrable<Url>,
    from: Deferrable<Url>,
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

impl<'a> TryProve<&'a Msg> for &'a Msg {
    type Error = (); // FIXME
    type Proven = &'a Msg;

    fn try_prove(self, proof: &'a Msg) -> Result<Self::Proven, ()> {
        if self == proof {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl<'a> TryProve<&'a Msg> for &'a MsgSend {
    type Error = (); // FIXME
    type Proven = &'a MsgSend;

    fn try_prove(self, proof: &'a Msg) -> Result<Self::Proven, ()> {
        if self.to == proof.to && self.from == proof.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

impl<'a> TryProve<&'a Msg> for &'a MsgReceive {
    type Error = (); // FIXME
    type Proven = &'a MsgReceive;

    fn try_prove(self, proof: &'a Msg) -> Result<Self::Proven, ()> {
        if self.to == proof.to && self.from == proof.from {
            Ok(self)
        } else {
            Err(())
        }
    }
}

// FIXME this needs to work on builders!
impl<'a> TryProve<&'a MsgReceive> for &'a MsgReceive {
    type Error = (); // FIXME
    type Proven = &'a MsgReceive;

    fn try_prove(self, proof: &'a MsgReceive) -> Result<Self::Proven, ()> {
        if self == proof {
            Ok(self)
        } else {
            Err(())
        }
    }
}
