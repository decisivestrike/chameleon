pub mod summary;
use std::collections::VecDeque;
pub use summary::Summary;

#[derive(Debug, Default)]
pub struct History {
    summaries: VecDeque<Summary>,
    max_len: usize,
}

impl History {
    pub fn new(max_len: usize) -> Self {
        Self {
            summaries: Default::default(),
            max_len,
        }
    }

    pub fn add(&mut self, summary: Summary) -> Option<Summary> {
        self.summaries.push_back(summary);

        if self.summaries.len() > self.max_len {
            self.summaries.pop_front()
        } else {
            None
        }
    }
}
