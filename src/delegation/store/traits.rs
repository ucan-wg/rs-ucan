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

    fn get_chain_cids(
        &self,
        audience: &DID,
        subject: &Option<DID>,
        command: String,
        policy: Vec<Predicate>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<Cid>>, Self::DelegationStoreError> {
        self.get_chain(audience, subject, command, policy, now)
            .map(|chain| chain.map(|chain| chain.map(|(cid, _)| cid)))
    }

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
            acc.push(self.get(cid)?);
            Ok(acc)
        })
    }
}

impl<T: Store<DID, V, C>, DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>>
    Store<DID, V, C> for &mut T
{
    type DelegationStoreError = <T as Store<DID, V, C>>::DelegationStoreError;

    fn get(&self, cid: &Cid) -> Result<&Delegation<DID, V, C>, Self::DelegationStoreError> {
        (**self).get(cid)
    }

    fn insert(
        &mut self,
        cid: Cid,
        delegation: Delegation<DID, V, C>,
    ) -> Result<(), Self::DelegationStoreError> {
        (**self).insert(cid, delegation)
    }

    fn revoke(&mut self, cid: Cid) -> Result<(), Self::DelegationStoreError> {
        (**self).revoke(cid)
    }

    fn get_chain(
        &self,
        audience: &DID,
        subject: &Option<DID>,
        command: String,
        policy: Vec<Predicate>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, &Delegation<DID, V, C>)>>, Self::DelegationStoreError> {
        (**self).get_chain(audience, subject, command, policy, now)
    }
}
