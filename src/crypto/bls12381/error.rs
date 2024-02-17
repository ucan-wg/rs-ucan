use blst::BLST_ERROR;
use enum_as_inner::EnumAsInner;
use thiserror::Error;

/// Errors that can occur during BLS verification.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Error, EnumAsInner)]
pub enum VerificationError {
    /// Signature mismatch.
    #[error("signature mismatch")]
    VerifyMsgFail,

    /// Bad encoding.
    #[error("bad encoding")]
    BadEncoding,

    /// Point not on curve.
    #[error("point not on curve")]
    PointNotOnCurve,

    /// Point not in group.
    #[error("bad point not in group")]
    PointNotInGroup,

    /// Aggregate type mismatch.
    #[error("aggregate type mismatch")]
    AggrTypeMismatch,

    /// Public key is infinity.
    #[error("public key is infinity")]
    PkIsInfinity,

    /// Bad scalar.
    #[error("bad scalar")]
    BadScalar,
}

impl TryFrom<BLST_ERROR> for VerificationError {
    type Error = ();

    fn try_from(err: BLST_ERROR) -> Result<Self, ()> {
        match err {
            BLST_ERROR::BLST_SUCCESS => Err(()),
            BLST_ERROR::BLST_VERIFY_FAIL => Ok(VerificationError::VerifyMsgFail),
            BLST_ERROR::BLST_BAD_ENCODING => Ok(VerificationError::BadEncoding),
            BLST_ERROR::BLST_POINT_NOT_ON_CURVE => Ok(VerificationError::PointNotOnCurve),
            BLST_ERROR::BLST_POINT_NOT_IN_GROUP => Ok(VerificationError::PointNotInGroup),
            BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => Ok(VerificationError::AggrTypeMismatch),
            BLST_ERROR::BLST_PK_IS_INFINITY => Ok(VerificationError::PkIsInfinity),
            BLST_ERROR::BLST_BAD_SCALAR => Ok(VerificationError::BadScalar),
        }
    }
}
