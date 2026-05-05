use tokio::io;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::error;

#[derive(Default)]
pub struct ProcessManager {
    tasks: Vec<Child>,
}

impl ProcessManager {
    pub async fn spawn(&mut self, mut cmd: Command) -> io::Result<()> {
        cmd.kill_on_drop(true);
        let child = cmd.spawn()?;
        self.tasks.push(child);

        Ok(())
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let task_count = self.tasks.len();

        if task_count == 0 {
            return Ok(());
        }

        let (tx, mut rx) = mpsc::channel(self.tasks.len());

        for mut child in self.tasks.drain(..) {
            let tx = tx.clone();

            tokio::spawn(async move {
                let status = child.wait().await;

                if let Err(e) = tx.send((child, status)).await {
                    error!("{}", e);
                }
            });
        }
        drop(tx);

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        loop {
            tokio::select! {
                Some((child, status)) = rx.recv() => {
                    println!("Процесс {} завершился со статусом: {:?}", child.id().unwrap_or(0), status);
                }
                _ = &mut ctrl_c => {
                    println!("Gracefull shutdown...");

                    for child in self.tasks.iter_mut() {
                        child.start_kill()?;
                        child.wait().await?;
                    }

                    break;
                }
            }
        }

        Ok(())
    }
}
