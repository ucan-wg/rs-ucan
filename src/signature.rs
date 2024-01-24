use libipld_core::link::Link;

pub struct Envelope<T>
where
    T: Capsule,
{
    pub sig: Box<[u8]>,
    pub payload: Link<T>,
}

pub enum Signature {
    One(Box<[u8]>),
    Batch {
        sig: Box<[u8]>,
        merkle_proof: Box<[u32]>,
    },
}

// TODO move to own module
pub trait Capsule {
    const TAG: &'static str;
}
