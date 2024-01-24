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

pub trait TryProven<'a, A> {
    type Proven1;
    type Error1;
    fn try_proven(&'a self, candidate: &'a A) -> Result<&'a Self::Proven1, Self::Error1>;
}

impl<'a, T, U> TryProven<'a, T> for U
where
    T: TryProve<'a, U>,
{
    type Proven1 = T::Proven;
    type Error1 = T::Error;

    fn try_proven(&'a self, candidate: &'a T) -> Result<&'a T::Proven, T::Error> {
        candidate.try_prove(self)
    }
}
