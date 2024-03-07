use super::Header;
use libipld_core::codec::Codec;

#[derive(Clone, Debug, PartialEq)]
pub struct Es256Header<C> {
    pub codec: C,
}

impl<C: TryFrom<u64>> TryFrom<&[u8]> for Es256Header<C> {
    type Error = (); // FIXME

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((0x1200, inner)) = unsigned_varint::decode::u16(&bytes) {
            if let Ok((0x12, more)) = unsigned_varint::decode::u8(&inner) {
                if let Ok((codec_info, &[])) = unsigned_varint::decode::u64(&more) {
                    let codec = C::try_from(codec_info).map_err(|_| ())?;
                    return Ok(Es256Header { codec });
                }
            }
        }

        Err(())
    }
}

impl<C: Into<u64>> From<Es256Header<C>> for Vec<u8> {
    fn from(es: Es256Header<C>) -> Vec<u8> {
        let mut tag_buf: [u8; 3] = Default::default();
        let tag = unsigned_varint::encode::u16(0x1200, &mut tag_buf);

        let mut hash_buf: [u8; 2] = Default::default();
        let hash = unsigned_varint::encode::u8(0x12, &mut hash_buf);

        let mut enc_buf: [u8; 10] = Default::default();
        let enc = unsigned_varint::encode::u64(es.codec.into(), &mut enc_buf);

        [tag, hash, enc].concat().into()
    }
}

impl<C: Codec + Into<u64> + TryFrom<u64>> Header<C> for Es256Header<C> {
    type Signature = p256::ecdsa::Signature;
    type Verifier = p256::ecdsa::VerifyingKey;

    fn codec(&self) -> &C {
        &self.codec
    }
}
