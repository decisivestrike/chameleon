use crate::{
    NotificationServer,
    notification::Notification,
    queue::NotificationQueue,
    server::{NotificationCommand, command::ReplaceCommand},
};
use gtk::glib;

pub struct NotificationManager {
    app: gtk::Application,
    queue: NotificationQueue,
}

impl NotificationManager {
    pub fn new(app: &gtk::Application) -> Self {
        Self {
            app: app.clone(),
            queue: NotificationQueue::new(),
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
        match command {
            NotificationCommand::Push(push_command) => {
                log::debug!("PUSH!");

                let id = push_command.id;
                let notification = Notification::new(&self.app, push_command);
                notification.show();

                if let Err(e) = self.queue.push(id, notification).await {
                    log::error!("{e}");
                }
            }
            NotificationCommand::Replace(replace_command) => {
                log::debug!("REPLACE!");

                let ReplaceCommand { id, summary, body } = replace_command;
                self.queue
                    .modify(id, |notification| {
                        notification.update(None, &summary, &body);
                    })
                    .await;
            }
        }
    }
}
