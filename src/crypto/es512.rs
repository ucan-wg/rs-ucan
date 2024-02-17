//! ES512 signature support (P-512)

use p521;
use signature::Verifier;
use std::fmt;

/// The verifying/public key for ES512.
#[derive(Clone)] // FIXME , Serialize, Deserialize)]
pub struct VerifyingKey(pub p521::ecdsa::VerifyingKey);

impl fmt::Debug for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VerifyingKey").finish()
    }
}

impl PartialEq for VerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_encoded_point(true) == other.0.to_encoded_point(true)
    }
}

impl Eq for VerifyingKey {}

impl Verifier<p521::ecdsa::Signature> for VerifyingKey {
    fn verify(
        &self,
        msg: &[u8],
        signature: &p521::ecdsa::Signature,
    ) -> Result<(), signature::Error> {
        self.0.verify(msg, &signature)
    }
}
