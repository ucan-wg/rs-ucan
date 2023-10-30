//! BLS12-381 signature support

use anyhow::anyhow;
use blst::BLST_ERROR;
use signature::SignatureEncoding;

use super::JWSSignature;

/// A BLS12-381 G1 signature
#[derive(Debug, Clone)]
pub struct Bls12381G1Sha256SswuRoNulSignature(pub blst::min_sig::Signature);

impl<'a> TryFrom<&'a [u8]> for Bls12381G1Sha256SswuRoNulSignature {
    type Error = BLST_ERROR;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(blst::min_sig::Signature::uncompress(bytes)?))
    }
}

impl From<Bls12381G1Sha256SswuRoNulSignature> for [u8; 48] {
    fn from(sig: Bls12381G1Sha256SswuRoNulSignature) -> Self {
        sig.0.compress()
    }
}

impl SignatureEncoding for Bls12381G1Sha256SswuRoNulSignature {
    type Repr = [u8; 48];
}

impl JWSSignature for Bls12381G1Sha256SswuRoNulSignature {
    const ALGORITHM: &'static str = "Bls12381G1";
}

/// A BLS12-381 G2 signature
#[derive(Debug, Clone)]
pub struct Bls12381G2Sha256SswuRoNulSignature(pub blst::min_pk::Signature);

impl<'a> TryFrom<&'a [u8]> for Bls12381G2Sha256SswuRoNulSignature {
    type Error = BLST_ERROR;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(blst::min_pk::Signature::uncompress(bytes)?))
    }
}

impl From<Bls12381G2Sha256SswuRoNulSignature> for [u8; 96] {
    fn from(sig: Bls12381G2Sha256SswuRoNulSignature) -> Self {
        sig.0.compress()
    }
}

impl SignatureEncoding for Bls12381G2Sha256SswuRoNulSignature {
    type Repr = [u8; 96];
}

impl JWSSignature for Bls12381G2Sha256SswuRoNulSignature {
    const ALGORITHM: &'static str = "Bls12381G2";
}

/// A verifier for BLS12-381 G1 signatures
#[cfg(feature = "bls-verifier")]
pub fn bls_12_381_g1_sha256_sswu_ro_nul_verifier(
    key: &[u8],
    payload: &[u8],
    signature: &[u8],
) -> Result<(), anyhow::Error> {
    let dst = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";
    let aug = &[];

    let key =
        blst::min_sig::PublicKey::uncompress(key).map_err(|_| anyhow!("invalid BLS12-381 key"))?;

    let signature = blst::min_sig::Signature::uncompress(signature)
        .map_err(|_| anyhow!("invalid BLS12-381 signature"))?;

    match signature.verify(true, payload, dst, aug, &key, true) {
        BLST_ERROR::BLST_SUCCESS => Ok(()),
        BLST_ERROR::BLST_BAD_ENCODING => Err(anyhow!("bad encoding")),
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => Err(anyhow!("point not on curve")),
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => Err(anyhow!("bad point not in group")),
        BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => Err(anyhow!("aggregate type mismatch")),
        BLST_ERROR::BLST_VERIFY_FAIL => Err(anyhow!("signature mismatch")),
        BLST_ERROR::BLST_PK_IS_INFINITY => Err(anyhow!("public key is infinity")),
        BLST_ERROR::BLST_BAD_SCALAR => Err(anyhow!("bad scalar")),
    }
}

/// A verifier for BLS12-381 G2 signatures
#[cfg(feature = "bls-verifier")]
pub fn bls_12_381_g2_sha256_sswu_ro_nul_verifier(
    key: &[u8],
    payload: &[u8],
    signature: &[u8],
) -> Result<(), anyhow::Error> {
    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
    let aug = &[];

    let key =
        blst::min_pk::PublicKey::uncompress(key).map_err(|_| anyhow!("invalid BLS12-381 key"))?;

    let signature = blst::min_pk::Signature::uncompress(signature)
        .map_err(|_| anyhow!("invalid BLS12-381 signature"))?;

    match signature.verify(true, payload, dst, aug, &key, true) {
        BLST_ERROR::BLST_SUCCESS => Ok(()),
        BLST_ERROR::BLST_BAD_ENCODING => Err(anyhow!("bad encoding")),
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => Err(anyhow!("point not on curve")),
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => Err(anyhow!("bad point not in group")),
        BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => Err(anyhow!("aggregate type mismatch")),
        BLST_ERROR::BLST_VERIFY_FAIL => Err(anyhow!("signature mismatch")),
        BLST_ERROR::BLST_PK_IS_INFINITY => Err(anyhow!("public key is infinity")),
        BLST_ERROR::BLST_BAD_SCALAR => Err(anyhow!("bad scalar")),
    }
}
