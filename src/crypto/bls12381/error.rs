use blst::BLST_ERROR;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerificationError {
    #[error("signature mismatch")]
    VerifyMsgFail,

    #[error("bad encoding")]
    BadEncoding,

    #[error("point not on curve")]
    PointNotOnCurve,

    #[error("bad point not in group")]
    PointNotInGroup,

    #[error("aggregate type mismatch")]
    AggrTypeMismatch,

    #[error("public key is infinity")]
    PkIsInfinity,

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
