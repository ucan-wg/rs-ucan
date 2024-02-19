use libipld_core::codec::{Codec, Encode};
use signature::Verifier;
use thiserror::Error;

pub trait Header<Enc: Codec + TryFrom<u32> + Into<u32>>:
    for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>>
{
    type Signature: signature::SignatureEncoding;
    type Verifier: signature::Verifier<Self::Signature>;

    fn codec(&self) -> &Enc;

    fn encode_payload<T: Encode<Enc>, Buf: std::io::Write>(
        &self,
        payload: T,
        buffer: &mut Buf,
    ) -> Result<(), libipld_core::error::Error> {
        payload.encode(Self::codec(self).clone(), buffer)
    }

    fn try_verify<'a, T: Encode<Enc>>(
        &self,
        verifier: &'a Self::Verifier,
        signature: &'a Self::Signature,
        payload: T,
    ) -> Result<(), VerifyError> {
        let mut buffer = vec![];
        self.encode_payload(payload, &mut buffer)
            .map_err(VerifyError::CodecError)?;

        verifier
            .verify(&buffer, signature)
            .map_err(VerifyError::SignatureError)
    }
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("Varsig codec error: {0}")]
    CodecError(libipld_core::error::Error),

    #[error("varsig signature error: {0}")]
    SignatureError(signature::Error),
}
