use crate::NotificationServer;
use crate::queue::NotificationQueue;
use crate::requests::NotificationData;
use gtkio::spawn_local;

pub struct NotificationManager {
    queue: NotificationQueue,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            queue: NotificationQueue::new(),
        }
    }

    pub fn run(self) {
        let (server, mut receiver) = NotificationServer::create();

        spawn_local(async move {
            loop {
                if let Some(message) = receiver.recv().await {
                    self.handle_command(message).await;
                }
            }
        });

        server.serve();
    }

    async fn handle_command(&self, data: NotificationData) {
        self.queue.push_or_replace(data).await;
    }
}
