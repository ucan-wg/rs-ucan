use crate::{ability::traits::Ability, prove::TryProve};
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, str::FromStr};
use url::Url;

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

                let from = if let Some(Ipld::String(from)) = map.get("from") {
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

impl Ability for MsgReceive {
    type Builder = MsgReceiveBuilder;
    const COMMAND: &'static str = "msg/receive";
}

impl TryFrom<&Ipld> for MsgReceive {
    type Error = (); // FIXME

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
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

                let from = if let Some(Ipld::String(from)) = map.get("from") {
                    Url::from_str(from).ok() // FIXME
                } else {
                    None
                };

                Ok(Self {
                    to: to.unwrap(),
                    from: from.unwrap(),
                })
            }
            _ => Err(()),
        }
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
