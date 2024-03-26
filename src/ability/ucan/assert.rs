use crate::ability::command::Command;
use crate::task::Task;
use libipld_core::cid::Cid;

// Things that you can assert include content and receipts

#[derive(Debug, PartialEq)]
pub struct Ran<T, E> {
    ran: Cid,
    out: Box<Result<T, E>>,
    next: Vec<Task>, // FIXME may be more than "just" a task
}

impl<T, E> Command for Ran<T, E> {
    const COMMAND: &'static str = "/ucan/ran";
}

///////////////
///////////////
///////////////

#[derive(Debug, PartialEq)]
pub struct Claim<T> {
    claim: T,
}

impl<T> Command for Claim<T> {
    const COMMAND: &'static str = "/ucan/claim";
}
