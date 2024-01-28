// NOTE must remain *un*exported!
pub(super) mod internal;
pub mod parentful;
pub mod parentless;
pub mod traits;

// #[cfg_attr(doc, aquamarine::aquamarine)]
// /// FIXME
// ///
// /// ```mermaid
// /// flowchart LR
// ///   Invocation --> more --> Self --> Proof --> more2
// ///   more[...]
// ///   more2[...]
// /// ```
