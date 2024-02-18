use crate::{
    delegation::{condition::Condition, Delegation},
    did::Did,
    proof::checkable::Checkable,
};
use libipld_core::cid::Cid;
use nonempty::NonEmpty;
use web_time::SystemTime;

// NOTE the T here is the builder... FIXME add one layer up and call T::Builder? May be confusing?
pub trait Store<B: Checkable, C: Condition, DID: Did> {
    type Error;

    fn get(&self, cid: &Cid) -> Result<&Delegation<B::Hierarchy, C, DID>, Self::Error>;

    // FIXME add a variant that calculated the CID from the capsulre header?
    // FIXME that means changing the name to insert_by_cid or similar
    // FIXME rename put
    fn insert(&mut self, cid: Cid, delegation: Delegation<B, C, DID>) -> Result<(), Self::Error>;

    // FIXME validate invocation
    // sore invocation
    // just... move to invocation
    fn revoke(&mut self, cid: Cid) -> Result<(), Self::Error>;

    fn get_chain(
        &self,
        audience: &DID,
        subject: &DID,
        builder: &B,
        conditions: Vec<C>,
        now: SystemTime,
    ) -> Result<Option<NonEmpty<(Cid, &Delegation<B::Hierarchy, C, DID>)>>, Self::Error>;

    fn can_delegate(
        &self,
        issuer: &DID,
        audience: &DID,
        builder: &B,
        conditions: Vec<C>,
        now: SystemTime,
    ) -> Result<bool, Self::Error> {
        self.get_chain(audience, issuer, builder, conditions, now)
            .map(|chain| chain.is_some())
    }

    fn get_many(
        &self,
        cids: &[Cid],
    ) -> Result<Vec<&Delegation<B::Hierarchy, C, DID>>, Self::Error> {
        cids.iter().try_fold(vec![], |mut acc, cid| {
            let d: &Delegation<B::Hierarchy, C, DID> = self.get(cid)?;
            acc.push(d);
            Ok(acc)
        })
    }
}
