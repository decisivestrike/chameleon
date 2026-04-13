use crate::NotificationServer;
use crate::queue::NotificationQueue;
use crate::server::NotificationCommand;
use gtk::glib;

pub struct NotificationManager {
    queue: NotificationQueue,
}

impl NotificationManager {
    pub fn new(app: &gtk::Application) -> Self {
        Self {
            queue: NotificationQueue::new(app),
        }
    }

    pub fn run(self) {
        let (server, mut receiver) = NotificationServer::new();

        glib::spawn_future_local(async move {
            loop {
                if let Some(message) = receiver.recv().await {
                    self.handle_command(message).await;
                }
            }
        });

        server.serve();
    }

    async fn handle_command(&self, command: NotificationCommand) {
        self.queue.push_or_replace(command).await;
    }
}
