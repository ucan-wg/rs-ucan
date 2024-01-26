use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, fmt::Debug};

pub trait Command {
    const COMMAND: &'static str;
}

// FIXME Delegable and make it proven?
pub trait Delegatable: Sized {
    type Builder: Debug + TryInto<Self> + From<Self>;
}

pub trait Resolvable: Sized {
    type Awaiting: Command + Debug + TryInto<Self> + From<Self>;
}

// FIXME Delegatable?
pub trait Runnable {
    type Output;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynJs {
    pub cmd: &'static str,
    pub args: BTreeMap<String, Ipld>,
}

impl Delegatable for DynJs {
    type Builder = Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsHack(pub DynJs);

impl From<DynJs> for JsHack {
    fn from(dyn_js: DynJs) -> Self {
        Self(dyn_js)
    }
}

impl From<JsHack> for Ipld {
    fn from(js_hack: JsHack) -> Self {
        let mut map = BTreeMap::new();
        map.insert("command".into(), js_hack.0.cmd.into());
        map.into()
    }
}

impl TryFrom<Ipld> for DynJs {
    type Error = (); // FIXME

    fn try_from(_ipld: Ipld) -> Result<Self, ()> {
        todo!()
    }
}

impl Command for JsHack {
    const COMMAND: &'static str = "ucan/dyn/js";
}

impl Delegatable for JsHack {
    type Builder = Self;
}
