use super::Header;
use crate::crypto::rs512::{Signature, VerifyingKey};
use libipld_core::codec::Codec;

#[derive(Clone, Debug, PartialEq)]
pub struct Rs512Header<C> {
    pub codec: C,
}

impl<C: TryFrom<u64>> TryFrom<&[u8]> for Rs512Header<C> {
    type Error = (); // FIXME

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((0x1205, inner)) = unsigned_varint::decode::u16(&bytes) {
            if let Ok((0x13, more)) = unsigned_varint::decode::u8(&inner) {
                if let Ok((codec_info, &[])) = unsigned_varint::decode::u64(&more) {
                    let codec = C::try_from(codec_info).map_err(|_| ())?;
                    return Ok(Rs512Header { codec });
                }
            }
        }

        Err(())
    }
}

impl<C: Into<u64>> From<Rs512Header<C>> for Vec<u8> {
    fn from(rs: Rs512Header<C>) -> Vec<u8> {
        let mut tag_buf: [u8; 3] = Default::default();
        let tag = unsigned_varint::encode::u16(0x1205, &mut tag_buf);

        let mut hash_buf: [u8; 2] = Default::default();
        let hash = unsigned_varint::encode::u8(0x13, &mut hash_buf);

        let mut enc_buf: [u8; 10] = Default::default();
        let enc = unsigned_varint::encode::u64(rs.codec.into(), &mut enc_buf);

        [tag, hash, enc].concat().into()
    }
}

impl<C: Codec + Into<u64> + TryFrom<u64>> Header<C> for Rs512Header<C> {
    type Signature = Signature;
    type Verifier = VerifyingKey;

    fn codec(&self) -> &C {
        &self.codec
    }
}
