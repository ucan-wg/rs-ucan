use crate::{
    crypto::varsig,
    delegation::{condition::Condition, Delegation},
    did::Did,
    //   proof::checkable::Checkable,
};
use libipld_core::{cid::Cid, codec::Codec};
use nonempty::NonEmpty;
use std::fmt::Debug;
use web_time::SystemTime;

// NOTE the T here is the builder... FIXME add one layer up and call T::Builder? May be confusing?
pub trait Store<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
>
{
    type DelegationStoreError: Debug;

    fn get(&self, cid: &Cid) -> Result<&Delegation<C, DID, V, Enc>, Self::DelegationStoreError>;

    // FIXME add a variant that calculated the CID from the capsulre header?
    // FIXME that means changing the name to insert_by_cid or similar
    // FIXME rename put
    fn insert(
        &mut self,
        cid: Cid,
        delegation: Delegation<C, DID, V, Enc>,
    ) -> Result<(), Self::DelegationStoreError>;

    // FIXME validate invocation
    // sore invocation
    // just... move to invocation
    fn revoke(&mut self, cid: Cid) -> Result<(), Self::DelegationStoreError>;

    fn get_chain(
        &self,
        audience: &DID,
        subject: &Option<DID>,
        policy: Vec<C>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, &Delegation<C, DID, V, Enc>)>>, Self::DelegationStoreError>;

    fn can_delegate(
        &self,
        issuer: DID,
        audience: &DID,
        policy: Vec<C>,
        now: SystemTime,
    ) -> Result<bool, Self::DelegationStoreError> {
        self.get_chain(audience, &Some(issuer), policy, now)
            .map(|chain| chain.is_some())
    }

    fn get_many(
        &self,
        cids: &[Cid],
    ) -> Result<Vec<&Delegation<C, DID, V, Enc>>, Self::DelegationStoreError> {
        cids.iter().try_fold(vec![], |mut acc, cid| {
            let d: &Delegation<C, DID, V, Enc> = self.get(cid)?;
            acc.push(d);
            Ok(acc)
        })
    }
}
