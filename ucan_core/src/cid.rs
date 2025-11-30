use ipld_core::cid::{multihash::Multihash, Cid};
use serde::Serialize;
use sha2::Digest;

/// Serialize a value to a DAG-CBOR/SHA2-256 CID.
pub fn to_dagcbor_cid<T: Serialize>(t: &T) -> Cid {
    let bytes = serde_ipld_dagcbor::to_vec(t).expect("delegation is not serializable");
    let digest = sha2::Sha256::digest(bytes);
    let multihash =
        Multihash::wrap(0x12, &digest).expect("unable to create multihash for delegation");
    ipld_core::cid::Cid::new_v1(0x71, multihash)
}
