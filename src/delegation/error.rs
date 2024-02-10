pub enum EnvelopeError {
    InvalidSubject,
    MisalignedIssAud,
    Expired,
    NotYetValid,
}

// FIXME Error, etc
pub enum DelegationError<Semantic> {
    Envelope(EnvelopeError),

    FailedCondition, // FIXME add context?

    SemanticError(Semantic), // // FIXME these are duplicated in Outcome
                             // //
                             // /// An error in the command chain.
                             // CommandEscelation,

                             // /// An error in the argument chain.
                             // ArgumentEscelation(ArgErr),

                             // /// An error in the proof chain.
                             // InvalidProofChain(ChainErr),

                             // /// An error in the parents.
                             // InvalidParents(ParentErr),
}

impl<S> From<EnvelopeError> for DelegationError<S> {
    fn from(err: EnvelopeError) -> Self {
        DelegationError::Envelope(err)
    }
}
