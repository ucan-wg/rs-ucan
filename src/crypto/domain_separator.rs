//! Domain separation utilities.

/// Static domain separator for the DID method.
pub trait DomainSeparator {
    /// The domain separator bytes;
    const DST: &'static [u8];
}
