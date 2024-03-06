/// A trait aligning signatures with keys.

use crate::crypto::{bls12381, es512, rs256, rs512};
use ::p521 as ext_p521;
use ed25519_dalek;
use k256;
use p256;
use p384;

// FIXME
// also: e.g. HSM?

pub trait DidKey: signature::Verifier<Self::Signature> {
    const BASE58_PREFIX: &'static str;

    type Signer: signature::Signer<Self::Signature>;
    type Signature: signature::SignatureEncoding;
}

impl DidKey for ed25519_dalek::VerifyingKey {
    const BASE58_PREFIX: &'static str = "6Mk";

    type Signer = ed25519_dalek::SigningKey;
    type Signature = ed25519_dalek::Signature;
}

impl DidKey for p256::ecdsa::VerifyingKey {
    const BASE58_PREFIX: &'static str = "Dn";

    type Signer = p256::ecdsa::SigningKey;
    type Signature = p256::ecdsa::Signature;
}

impl DidKey for k256::ecdsa::VerifyingKey {
    const BASE58_PREFIX: &'static str = "Q3s";

    type Signer = k256::ecdsa::SigningKey;
    type Signature = k256::ecdsa::Signature;
}

impl DidKey for p384::ecdsa::VerifyingKey {
    const BASE58_PREFIX: &'static str = "82";

    type Signer = p384::ecdsa::SigningKey;
    type Signature = p384::ecdsa::Signature;
}

impl DidKey for es512::VerifyingKey {
    const BASE58_PREFIX: &'static str = "2J9";

    type Signer = ext_p521::ecdsa::SigningKey;
    type Signature = ext_p521::ecdsa::Signature;
}

impl DidKey for rs256::VerifyingKey {
    const BASE58_PREFIX: &'static str = "4MX";

    type Signer = rs256::SigningKey;
    type Signature = rs256::Signature;
}

impl DidKey for rs512::VerifyingKey {
    const BASE58_PREFIX: &'static str = "zgg";

    type Signer = rs512::SigningKey;
    type Signature = rs512::Signature;
}

impl DidKey for blst::min_sig::PublicKey {
    const BASE58_PREFIX: &'static str = "UC7";

    type Signer = blst::min_sig::SecretKey;
    type Signature = bls12381::min_sig::Signature;
}

impl DidKey for blst::min_pk::PublicKey {
    const BASE58_PREFIX: &'static str = "UC7";

    type Signer = blst::min_pk::SecretKey;
    type Signature = bls12381::min_pk::Signature;
}
