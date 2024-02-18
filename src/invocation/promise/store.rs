//! Storage of resolved and unresolved promises.

mod memory;
mod traits;

pub use memory::MemoryStore;
pub use traits::Store;
