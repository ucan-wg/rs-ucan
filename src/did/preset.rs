use super::key;
use enum_as_inner::EnumAsInner;

/// The set of [`Did`] types that ship with this library ("presets").
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum Verifier {
    /// `did:key` DIDs.
    Key(key::Verifier),
    // Dns(did_url::DID),
}

// FIXME serialize with did:key etc

impl signature::Verifier<key::Signature> for Verifier {
    fn verify(&self, message: &[u8], signature: &key::Signature) -> Result<(), signature::Error> {
        match self {
            Verifier::Key(verifier) => verifier.verify(message, signature),
        }
    }
}
