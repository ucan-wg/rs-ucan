#[cfg_attr(doc, aquamarine::aquamarine)]
/// FIXME
///
/// ```mermaid
/// flowchart LR
///   Invocation --> more --> Self --> Candidate --> more2
///   more[...]
///   more2[...]
/// ```
pub trait TryProve<'a, T> {
    type Error;
    type Proven;

    fn try_prove(&'a self, candidate: &'a T) -> Result<&'a Self::Proven, Self::Error>;
}
