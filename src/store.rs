//! A store for persisting UCAN tokens, to be referencable as proofs by other UCANs

use std::{collections::HashMap, io::Cursor, marker::PhantomData};

use async_trait::async_trait;
use cid::{multihash, Cid};
use libipld_core::{
    codec::{Codec, Decode, Encode},
    raw::RawCodec,
};
use multihash::MultihashDigest;

use crate::DEFAULT_MULTIHASH;

/// A store for persisting UCAN tokens, to be referencable as proofs by other UCANs
pub trait Store<C>
where
    C: Codec,
{
    /// The error type for this store
    type Error;

    /// Read a token from the store
    fn read<T>(&self, cid: &Cid) -> Result<Option<T>, Self::Error>
    where
        T: Decode<C>;

    /// Write a token to the store, using the specified hasher
    fn write<T>(&mut self, token: T, hasher: Option<multihash::Code>) -> Result<Cid, Self::Error>
    where
        T: Encode<C>;
}

/// An async store for persisting UCAN tokens, to be referencable as proofs by other UCANs
// TODO: The send / sync bounds need to be conditional based on the target, to support wasm32
#[async_trait]
pub trait AsyncStore<C>: Send + Sync
where
    C: Codec,
{
    /// The error type for this store
    type Error;

    /// Read a token from the store
    async fn read<T>(&self, cid: &Cid) -> Result<Option<T>, Self::Error>
    where
        T: Decode<C>;

    /// Write a token to the store, using the specified hasher
    async fn write<T>(
        &mut self,
        token: T,
        hasher: Option<multihash::Code>,
    ) -> Result<Cid, Self::Error>
    where
        T: Encode<C> + Send;
}

/// An in-memory store for development and testing
#[derive(Debug, Clone, Default)]
pub struct InMemoryStore<C> {
    store: HashMap<Cid, Vec<u8>>,
    _phantom: PhantomData<C>,
}

impl Store<RawCodec> for InMemoryStore<RawCodec> {
    type Error = anyhow::Error;

    fn read<T>(&self, cid: &Cid) -> Result<Option<T>, Self::Error>
    where
        T: Decode<RawCodec>,
    {
        match self.store.get(cid) {
            Some(block) => Ok(Some(T::decode(RawCodec, &mut Cursor::new(block))?)),
            None => Ok(None),
        }
    }

    fn write<T>(&mut self, token: T, hasher: Option<multihash::Code>) -> Result<Cid, Self::Error>
    where
        T: Encode<RawCodec>,
    {
        let hasher = hasher.unwrap_or(DEFAULT_MULTIHASH);
        let block = RawCodec.encode(&token)?;
        let digest = hasher.digest(&block);
        let cid = Cid::new_v1(RawCodec.into(), digest);

        self.store.insert(cid, block);

        Ok(cid)
    }
}
