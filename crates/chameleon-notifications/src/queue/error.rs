use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Notification #{0} already exists")]
    DuplicateId(u32),
}
