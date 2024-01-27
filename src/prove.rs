#[cfg_attr(doc, aquamarine::aquamarine)]
/// FIXME
///
/// ```mermaid
/// flowchart LR
///   Invocation --> more --> Self --> Candidate --> more2
///   more[...]
///   more2[...]
/// ```
pub trait TryProve<T> {
    type Error;
    type Proven;

    fn try_prove(self, candidate: T) -> Result<Self::Proven, Self::Error>;
}
