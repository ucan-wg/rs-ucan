use super::Header;
use libipld_core::codec::Codec;

#[derive(Clone, Debug, PartialEq)]
pub struct EdDsaHeader<C> {
    pub codec: C,
}

impl<C: TryFrom<u32>> TryFrom<&[u8]> for EdDsaHeader<C> {
    type Error = (); // FIXME

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((0xed, inner)) = unsigned_varint::decode::u8(&bytes) {
            if let Ok((codec_info, &[])) = unsigned_varint::decode::u32(&inner) {
                let codec = C::try_from(codec_info).map_err(|_| ())?;
                return Ok(EdDsaHeader { codec });
            }
        }

        return Err(());
    }
}

impl<C: Into<u32> + Clone> From<EdDsaHeader<C>> for Vec<u8> {
    fn from(ed: EdDsaHeader<C>) -> Vec<u8> {
        let mut tag_buf: [u8; 2] = Default::default();
        let tag: &[u8] = unsigned_varint::encode::u8(0xed, &mut tag_buf);

        let mut enc_buf: [u8; 5] = Default::default();
        let enc: &[u8] = unsigned_varint::encode::u32(ed.codec.into(), &mut enc_buf);

        [tag, enc].concat().into()
    }
}

impl<C: Codec + Into<u32> + TryFrom<u32>> Header<C> for EdDsaHeader<C> {
    type Signature = ed25519_dalek::Signature;
    type Verifier = ed25519_dalek::VerifyingKey;

    fn codec(&self) -> &C {
        &self.codec
    }
}
