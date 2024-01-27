use std::convert::Infallible;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// FIXME
///
/// ```mermaid
/// flowchart LR
///   Invocation --> more --> Self --> Proof --> more2
///   more[...]
///   more2[...]
/// ```
pub trait TryProve<T: ?Sized> {
    type Proven;
    type Error;

    // FIXME rename to proof?
    fn try_prove(&self, proof: T) -> Result<Self::Proven, Self::Error>;
}

// pub trait Prove<T> {
//     type Proven;
//
//     fn prove(self, proof: T) -> Self::Proven;
// }
//
// impl<T: Prove<U> + ?Sized, U> TryProve<U> for T {
//     type Proven = T::Proven;
//     type Error = Infallible;
//
//     fn try_prove(&self, proof: U) -> Result<Self::Proven, Infallible> {
//         Ok(self.prove(proof))
//     }
// }
