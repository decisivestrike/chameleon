use crate::NotificationServer;
use crate::config::Rules;
use crate::history::History;
use crate::server::Action;
use crate::window::NotificationWindow;
use crate::windows_map::WindowsMap;
use arc_swap::ArcSwap;
use gtkio::future::spawn_local;

pub struct NotificationManager {
    history: History,
    windows_map: WindowsMap,
    windows_factory: WindowsFactory,
    rules: ArcSwap<Rules>,
}

impl NotificationManager {
    pub fn new(rules: ArcSwap<Rules>) -> Self {
        Self {
            history: History::default(),
            windows_map: WindowsMap::default(),
            rules,
        }
    }

    pub fn run(self) {
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

    async fn handle_action(&self, action: Action) {
        match action {
            Action::Create(data) => {
                if self.windows_map.contains_id(data.id) {
                    self.windows_map.modify(data.id, |window| {});
                } else {
                    let window = NotificationWindow::new();

                    self.windows_map.insert(
                        data.id,
                        window,
                        data.expire_timeout.into(),
                    );
                }

                // add to history
                // history.add(data.summarize())
            }
            Action::Remove(id) => {
                self.windows_map.remove(id);
            }
        }
    }
}
