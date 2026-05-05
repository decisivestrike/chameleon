use crate::NotificationServer;
use crate::config::Rules;
use crate::queue::NotificationQueue;
use crate::requests::NotificationData;
use crate::windows_factory::WindowsFactory;
use gtkio::future::spawn_local;

pub struct NotificationManager {
    history: Vec<NotificationSummary>,
    queue: NotificationQueue,
}

impl NotificationManager {
    pub fn new(config: Rules) -> Self {
        let Rules {
            queue_config,
            window_rules: window_config,
        } = config;

        let factory = WindowsFactory::new(window_config);

        Self {
            queue: NotificationQueue::new(queue_config, factory),
        }
    }

    pub fn run(self) {
        let (server, mut receiver) = NotificationServer::create();

        spawn_local(async move {
            loop {
                if let Some(notification) = receiver.recv().await {
                    self.handle_notification(&notification).await;
                }
            }
        });

        server.serve();
    }

    async fn handle_notification(&self, data: &NotificationData) {
        self.queue.push_or_replace(data).await;
    }
}
