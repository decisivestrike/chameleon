use crate::NotificationServer;
use crate::config::Rules;
use crate::notification::NotificationData;
use crate::queue::NotificationQueue;

use arc_swap::ArcSwap;
use gtkio::future::spawn_local;

pub struct NotificationManager {
    history: Vec<NotificationSummary>,
    queue: NotificationQueue,
    rules: ArcSwap<Rules>,
}

impl NotificationManager {
    pub fn new(config: Rules) -> Self {
        let Rules {
            queue_config,
            window_rules: window_config,
        } = config;

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
        // create window

        // add to history

        self.queue.push_or_replace(data).await;
    }
}
