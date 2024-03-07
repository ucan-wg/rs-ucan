use crate::crypto::signature::Envelope;
use crate::delegation::Delegation;
use libipld_core::codec::Codec;
use libipld_core::codec::Encode;
use libipld_core::ipld::Ipld;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Preset {
    Identity = 0x5f,
    DagPb = 0x70,
    DagCbor = 0x71,
    DagJson = 0x0129,
    Jwt = 0x6a77, // FIXME break out Jwt & EIP-191?
    Eip191 = 0xe191,
}

impl Encode<Preset> for Ipld {
    fn encode<W: std::io::Write>(
        &self,
        c: Preset,
        w: &mut W,
    ) -> Result<(), libipld_core::error::Error> {
        match c {
            Preset::Identity => todo!(),
            Preset::DagPb => todo!(),
            Preset::DagCbor => self.encode(libipld_cbor::DagCborCodec, w),
            Preset::DagJson => todo!(),
            Preset::Jwt => todo!(),
            Preset::Eip191 => todo!(),
        }
    }
}

impl Encode<Preset> for Delegation {
    fn encode<W: std::io::Write>(
        &self,
        c: Preset,
        w: &mut W,
    ) -> Result<(), libipld_core::error::Error> {
        self.clone().to_ipld_envelope().encode(c, w)
    }
}

impl TryFrom<u64> for Preset {
    type Error = libipld_core::error::UnsupportedCodec;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x5f => Ok(Preset::Identity),
            0x70 => Ok(Preset::DagPb),
            0x71 => Ok(Preset::DagCbor),
            0x0129 => Ok(Preset::DagJson),
            0x6a77 => Ok(Preset::Jwt),
            0xe191 => Ok(Preset::Eip191),
            // 0xe1 => Ok(Preset::MerkleBatchSig),
            _ => Err(libipld_core::error::UnsupportedCodec(value)),
        }
    }
}

impl From<Preset> for u64 {
    fn from(encoding: Preset) -> u64 {
        encoding as u64
    }
}

impl Codec for Preset {}

// FIXME pub struct MerkleSig

impl<'a> TryFrom<&'a [u8]> for Preset {
    type Error = ();

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if let (encoding_info, &[]) = unsigned_varint::decode::u64(&bytes).map_err(|_| ())? {
            return match encoding_info {
                0x5f => Ok(Preset::Identity),
                0x70 => Ok(Preset::DagPb),
                0x71 => Ok(Preset::DagCbor),
                0x0129 => Ok(Preset::DagJson),
                0x6a77 => Ok(Preset::Jwt),
                0xe191 => Ok(Preset::Eip191),
                // 0xe1 => {
                //     let merkle_proof = Vec::new();
                //     Ok(Preset::MerkleBatchSig(merkle_proof))
                // }
                _ => Err(()),
            };
        };

        Err(())
    }
}

impl AsRef<[u8]> for Preset {
    fn as_ref(&self) -> &[u8] {
        match self {
            Preset::Identity => &[0x5f],
            Preset::DagPb => &[0x70],
            Preset::DagCbor => &[0x71],
            Preset::DagJson => &[0x01, 0x29],
            Preset::Jwt => &[0x6a, 0x77],
            Preset::Eip191 => &[0xe1, 0x91],
            // Preset::Eip191(inner) => {
            //     let mut buffer = vec![0xe191];
            //     buffer.extend(inner.as_ref());
            //     buffer.as_ref()
            // } // Preset::MerkleBatchSig(merkle_proof) => {
            //     let mut buffer = vec![0xe1];
            //     buffer.extend(merkle_proof.as_ref());
            //     buffer.as_ref()
            // }
        }
    }
}

impl From<Preset> for u32 {
    fn from(encoding: Preset) -> u32 {
        match encoding {
            Preset::Identity => 0x5f,
            Preset::DagPb => 0x70,
            Preset::DagCbor => 0x71,
            Preset::DagJson => 0x0129,
            Preset::Jwt => 0x6a77,
            Preset::Eip191 => 0xe191,
            // Preset::MerkleBatchSig(_) => 0xe1,
        }
    }
}

impl TryFrom<u32> for Preset {
    type Error = libipld_core::error::UnsupportedCodec;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x5f => Ok(Preset::Identity),
            0x70 => Ok(Preset::DagPb),
            0x71 => Ok(Preset::DagCbor),
            0x0129 => Ok(Preset::DagJson),
            0x6a77 => Ok(Preset::Jwt),
            0xe191 => Ok(Preset::Eip191),
            // 0xe1 => Ok(Preset::MerkleBatchSig),
            _ => Err(libipld_core::error::UnsupportedCodec(value as u64)),
        }
    }
}
