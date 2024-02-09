use super::error::OptionalFieldError;

pub fn check_optional<T: PartialEq>(
    target: Option<T>,
    proof: Option<T>,
) -> Result<(), OptionalFieldError> {
    if let Some(target_value) = target {
        let proof_value = proof.ok_or(OptionalFieldError::Missing)?;
        if target_value != proof_value {
            return Err(OptionalFieldError::Unequal);
        }
    }

    Ok(())
}
