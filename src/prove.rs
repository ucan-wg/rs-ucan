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

    fn try_prove<'a>(&'a self, candidate: &'a T) -> Result<&'a Self::Proven, Self::Error>;
}

pub trait TryProven<A> {
    type Proven1;
    type Error1;
    fn try_proven<'a>(&'a self, candidate: &'a A) -> Result<&'a Self::Proven1, Self::Error1>;
}

impl<T, U> TryProven<T> for U
where
    T: TryProve<U>,
{
    type Proven1 = T::Proven;
    type Error1 = T::Error;

    fn try_proven<'a>(&'a self, candidate: &'a T) -> Result<&'a T::Proven, T::Error> {
        candidate.try_prove(self)
    }
}
