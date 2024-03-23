//! Storage for [`Invocation`]s.

mod memory;
mod traits;

pub use memory::{MemoryStore, MemoryStoreInner};
pub use traits::Store;
