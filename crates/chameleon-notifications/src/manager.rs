use crate::NotificationServer;
use crate::config::Rules;
use crate::history::{History, Summary};
use crate::server::Action;
use crate::window::{NotificationContent, NotificationWindow};
use crate::windows_map::WindowsMap;
use gtkio::future::spawn_local;

pub struct NotificationManager {
    history: History,
    windows_map: WindowsMap,
    rules: Rules,
}

impl NotificationManager {
    pub fn new(rules: Rules) -> Self {
        Self {
            history: History::new(rules.max_history),
            windows_map: WindowsMap::new(
                rules.max_active.get().into(),
                rules.spacing.into(),
            ),
            rules,
        }
    }

    pub fn run(mut self) {
        let (server, mut receiver) = NotificationServer::create();

        spawn_local(async move {
            loop {
                if let Some(action) = receiver.recv().await {
                    self.handle_action(action).await;
                }
            }
        });

        server.serve();
    }

    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::Create(data) => {
                if self.windows_map.contains_id(data.id) {
                    self.windows_map.modify(data.id, |window| {
                        let content = NotificationContent::new(
                            &data,
                            &self.rules.window.content,
                        );
                        window.set_content(content);
                    });
                } else {
                    let content = NotificationContent::new(
                        &data,
                        &self.rules.window.content,
                    );
                    let window =
                        NotificationWindow::new(content, &self.rules.window);

                    self.windows_map.insert(
                        data.id,
                        window,
                        data.expire_timeout.into(),
                    );
                }

                if let Some(summary) = Summary::from_data(data) {
                    self.history.add(summary);
                }
            }
            Action::Remove(id) => {
                self.windows_map.remove(id);
            }
        }
    }
}
