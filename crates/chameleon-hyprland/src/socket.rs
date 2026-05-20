use crate::HyprEvent;
use anyhow::Result;
use std::path::Path;
use tokio::io::{self, AsyncBufReadExt, BufReader, Lines};
use tokio::net::UnixStream;

pub struct Socket {
    lines: Lines<BufReader<UnixStream>>,
}

impl Socket {
    pub async fn connect_to(path: impl AsRef<Path>) -> io::Result<Self> {
        let stream = UnixStream::connect(path.as_ref()).await?;
        let reader = BufReader::new(stream);
        let lines = reader.lines();

        Ok(Self { lines })
    }

    pub async fn wait_event(&mut self) -> Result<Option<HyprEvent>> {
        let maybe_line = self.lines.next_line().await?;
        let result = maybe_line.map(|line| line.parse()).transpose()?;

        Ok(result)
    }
}
