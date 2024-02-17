// FIXME rename this is not for the sign envelope
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeError {
    InvalidSubject,
    MisalignedIssAud,
    Expired,
    NotYetValid,
}

// FIXME Error, etc
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationError<Semantic> {
    Envelope(EnvelopeError),

    FailedCondition, // FIXME add context?

    SemanticError(Semantic),
}

impl<S> From<EnvelopeError> for DelegationError<S> {
    fn from(err: EnvelopeError) -> Self {
        DelegationError::Envelope(err)
    }
}
