//! Check the delegation proof against another instance of the same type

use super::error::{OptionalFieldError, Unequal};
use crate::did::Did;

/// Trait for checking if a proof of the same type is equally or less restrictive.
///
/// # Example
///
/// ```rust
/// # use ucan::proof::same::CheckSame;
/// # use ucan::did::Did;
/// #
/// struct HelloBuilder {
///    wave_at: Option<Did>,
/// }
///
/// enum HelloError {
///   MissingWaveAt,
///   WeDontTalkTo(Did)
/// }
///
/// impl CheckSame for HelloBuilder {
///     type Error = HelloError;
///
///     fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
///         if self.wave_at == Some(Did::try_from("did:example:mallory".to_string()).unwrap()) {
///             return Err(HelloError::WeDontTalkTo(self.wave_at.clone().unwrap()));
///         }
///
///         if let Some(_) = &proof.wave_at {
///             if self.wave_at != proof.wave_at {
///                 return Err(HelloError::MissingWaveAt);
///             }
///         }
///
///         Ok(())
///     }
/// }
pub trait CheckSame {
    /// Error type describing why a proof was insufficient.
    type Error; // FIXME Rename CheckSameError?

    /// Check if the proof is equally or less restrictive than the instance.
    ///
    /// Delegation must always attenuate. If the proof is more restrictive than the instance,
    /// it has violated the delegation chain rules.
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error>;
}

impl CheckSame for Did {
    type Error = Unequal;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.eq(proof) {
            Ok(())
        } else {
            Err(Unequal {})
        }
    }
}

impl<T: PartialEq + Clone> CheckSame for Option<T> {
    type Error = OptionalFieldError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match proof {
            None => Ok(()),
            Some(proof_) => match self {
                None => Err(OptionalFieldError::Missing),
                Some(self_) => {
                    if self_.eq(proof_) {
                        Ok(())
                    } else {
                        Err(OptionalFieldError::Unequal)
                    }
                }
            },
        }
    }
}
