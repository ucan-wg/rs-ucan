//! Message abilities

mod any;
mod receive;

pub mod send;

pub use any::Any;
pub use receive::Receive;

// FIXME rename invokable?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ready {
    Receive(receive::Receive),
    Send(send::Ready),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Builder {
    Recieve(receive::Receive),
    Send(send::Builder),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    // Recieve(receive::Promised), // FIXME
    Send(send::Promised),
}
