use super::Header;
use libipld_core::codec::Codec;

#[derive(Clone, Debug, PartialEq)]
pub struct Es256kHeader<C> {
    pub codec: C,
}

impl<C: TryFrom<u32>> TryFrom<&[u8]> for Es256kHeader<C> {
    type Error = (); // FIXME

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((0xe7, inner)) = unsigned_varint::decode::u8(&bytes) {
            if let Ok((0x12, more)) = unsigned_varint::decode::u8(&inner).map_err(|_| ()) {
                if let Ok((codec_info, &[])) = unsigned_varint::decode::u32(&more) {
                    let codec = C::try_from(codec_info).map_err(|_| ())?;
                    return Ok(Es256kHeader { codec });
                }
            }
        }

        Err(())
    }
}

impl<C: Into<u32>> From<Es256kHeader<C>> for Vec<u8> {
    fn from(es: Es256kHeader<C>) -> Vec<u8> {
        let mut tag_buf: [u8; 2] = Default::default();
        let tag = unsigned_varint::encode::u8(0xe7, &mut tag_buf);

        let mut hash_buf: [u8; 2] = Default::default();
        let hash = unsigned_varint::encode::u8(0x12, &mut hash_buf);

        let mut enc_buf: [u8; 5] = Default::default();
        let enc = unsigned_varint::encode::u32(es.codec.into(), &mut enc_buf);

        [tag, hash, enc].concat().into()
    }
}

impl<C: Codec + Into<u32> + TryFrom<u32>> Header<C> for Es256kHeader<C> {
    type Signature = k256::ecdsa::Signature;
    type Verifier = k256::ecdsa::VerifyingKey;

    fn codec(&self) -> &C {
        &self.codec
    }
}
