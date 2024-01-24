// pub struct ReceiptPayload<T> {
//     pub issuer: Did,
//     pub ran: Link<Invocation<T>>,
//     pub out: UcanResult<T>, // FIXME?
//     pub proofs: Vec<Link<Delegation<FIXME>>>,
//     pub metadata: BTreeMap<str, Ipld>, // FIXME serde value instead?
//     pub issued_at: u64,
// }

//
// pub enum UcanResult<T> {
//     UcanOk(T),
//     UcanErr(BTreeMap<&str, Ipld>),
// }
//
// pub struct Capability<T> {
//     command: String,
//     payload: BTreeMap<str, T>,
// }
