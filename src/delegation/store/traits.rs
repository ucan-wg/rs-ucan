use crate::{
    crypto::varsig,
    delegation::{policy::Predicate, Delegation},
    did::Did,
};
use libipld_core::{cid::Cid, codec::Codec};
use nonempty::NonEmpty;
use std::fmt::Debug;
use web_time::SystemTime;

pub trait Store<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u64> + Into<u64>> {
    type DelegationStoreError: Debug;

    fn get(&self, cid: &Cid) -> Result<&Delegation<DID, V, Enc>, Self::DelegationStoreError>;

    fn insert(
        &mut self,
        cid: Cid,
        delegation: Delegation<DID, V, Enc>,
    ) -> Result<(), Self::DelegationStoreError>;

    // FIXME validate invocation
    // store invocation
    // just... move to invocation
    fn revoke(&mut self, cid: Cid) -> Result<(), Self::DelegationStoreError>;

    fn get_chain(
        &self,
        audience: &DID,
        subject: &Option<DID>,
        command: String,
        policy: Vec<Predicate>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, &Delegation<DID, V, Enc>)>>, Self::DelegationStoreError>;

    fn can_delegate(
        &self,
        issuer: DID,
        audience: &DID,
        command: String,
        policy: Vec<Predicate>,
        now: SystemTime,
    ) -> Result<bool, Self::DelegationStoreError> {
        self.get_chain(audience, &Some(issuer), command, policy, now)
            .map(|chain| chain.is_some())
    }

    fn get_many(
        &self,
        cids: &[Cid],
    ) -> Result<Vec<&Delegation<DID, V, Enc>>, Self::DelegationStoreError> {
        cids.iter().try_fold(vec![], |mut acc, cid| {
            let d: &Delegation<DID, V, Enc> = self.get(cid)?;
            acc.push(d);
            Ok(acc)
        })
    }
}
