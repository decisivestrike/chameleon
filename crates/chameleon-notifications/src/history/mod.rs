pub mod summary;
pub use summary::Summary;

#[derive(Debug, Default)]
pub struct History {
    summaries: Vec<Summary>,
}
