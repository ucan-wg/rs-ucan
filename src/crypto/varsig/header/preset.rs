use super::{eddsa, es256, es256k, es512, rs256, rs512, Header};
use crate::{crypto::varsig::encoding, did::key};

#[derive(Clone, Debug, PartialEq)]
pub enum Preset {
    EdDsa(eddsa::EdDsaHeader<encoding::Preset>),
    Es256(es256::Es256Header<encoding::Preset>),
    Es256k(es256k::Es256kHeader<encoding::Preset>),
    Es512(es512::Es512Header<encoding::Preset>),
    Rs256(rs256::Rs256Header<encoding::Preset>),
    Rs512(rs512::Rs512Header<encoding::Preset>),
    // FIXME BLS? needs varsig specs
    // FIXME Es384 needs varsig specs
}

impl From<eddsa::EdDsaHeader<encoding::Preset>> for Preset {
    fn from(ed: eddsa::EdDsaHeader<encoding::Preset>) -> Self {
        Preset::EdDsa(ed)
    }
}

impl From<rs256::Rs256Header<encoding::Preset>> for Preset {
    fn from(rs256: rs256::Rs256Header<encoding::Preset>) -> Self {
        Preset::Rs256(rs256)
    }
}

impl From<rs512::Rs512Header<encoding::Preset>> for Preset {
    fn from(rs512: rs512::Rs512Header<encoding::Preset>) -> Self {
        Preset::Rs512(rs512)
    }
}

impl From<es256::Es256Header<encoding::Preset>> for Preset {
    fn from(es256: es256::Es256Header<encoding::Preset>) -> Self {
        Preset::Es256(es256)
    }
}

impl From<es256k::Es256kHeader<encoding::Preset>> for Preset {
    fn from(es256k: es256k::Es256kHeader<encoding::Preset>) -> Self {
        Preset::Es256k(es256k)
    }
}

impl From<Preset> for Vec<u8> {
    fn from(preset: Preset) -> Vec<u8> {
        match preset {
            Preset::EdDsa(ed) => ed.into(),
            Preset::Rs256(rs256) => rs256.into(),
            Preset::Rs512(rs512) => rs512.into(),
            Preset::Es256(es256) => es256.into(),
            Preset::Es256k(es256k) => es256k.into(),
            Preset::Es512(es512) => es512.into(),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Preset {
    type Error = ();

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if let Ok(ed) = eddsa::EdDsaHeader::try_from(bytes) {
            return Ok(Preset::EdDsa(ed));
        }

        if let Ok(rs256) = rs256::Rs256Header::<encoding::Preset>::try_from(bytes) {
            return Ok(Preset::Rs256(rs256));
        }

        if let Ok(rs512) = rs512::Rs512Header::<encoding::Preset>::try_from(bytes) {
            return Ok(Preset::Rs512(rs512));
        }

        if let Ok(es256) = es256::Es256Header::<encoding::Preset>::try_from(bytes) {
            return Ok(Preset::Es256(es256));
        }

        if let Ok(es256k) = es256k::Es256kHeader::<encoding::Preset>::try_from(bytes) {
            return Ok(Preset::Es256k(es256k));
        }

        if let Ok(es512) = es512::Es512Header::<encoding::Preset>::try_from(bytes) {
            return Ok(Preset::Es512(es512));
        }

        Err(())
    }
}

impl Header<encoding::Preset> for Preset {
    type Signature = key::Signature;
    type Verifier = key::Verifier;

    fn codec(&self) -> &encoding::Preset {
        match self {
            Preset::EdDsa(ed) => ed.codec(),
            Preset::Rs256(rs256) => rs256.codec(),
            Preset::Rs512(rs512) => rs512.codec(),
            Preset::Es256(es256) => es256.codec(),
            Preset::Es256k(es256k) => es256k.codec(),
            Preset::Es512(es512) => es512.codec(),
            // Preset::Bls
        }
    }
}
