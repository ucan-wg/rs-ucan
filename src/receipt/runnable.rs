use crate::{did::Did, nonce::Nonce, task, task::Task};

pub trait Runnable {
    type Success;

    fn to_task(&self, subject: Did, nonce: Nonce) -> Task;

    fn to_task_id(&self, subject: Did, nonce: Nonce) -> task::Id {
        task::Id {
            cid: self.to_task(subject, nonce).into(),
        }
    }

    // fn lookup(id: TaskId>) -> Result<Self::Output, ()>;
}