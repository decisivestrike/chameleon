//! https://specifications.freedesktop.org/notification/latest/protocol.html

use crate::NOTIFICATION_QUEUE;
use crate::factory::NotificationFactory;
use crate::requests::NotificationData;
use crate::responses::ServerInfo;
use grapes::tokio::sync::broadcast;
use grapes::tokio::task::JoinHandle;
use grapes::{RT, glib, gtk};
use std::future::{self};
use std::sync::Arc;
use zbus::connection::Builder as ConnectionBuilder;
use zbus::interface;

pub type Message = Arc<(u32, NotificationData)>;

pub struct NotificationServer {
    notification_id: u32,
    sender: broadcast::Sender<Message>,
}

impl NotificationServer {
    pub fn new() -> Self {
        Self {
            notification_id: 1,
            sender: broadcast::Sender::new(64),
        }
    }

    pub fn run(self, app: &gtk::Application) {
        let notification_factory = NotificationFactory::new(&app);

        self.on_message(move |message: Message| {
            let (id, data) = &*message;
            let notification = notification_factory.create(*id, &data);
            notification.show();

            async move {
                NOTIFICATION_QUEUE.append(notification).await;
            }
        });

        self.start_server();
    }

    fn on_message<Callback, F>(&self, f: Callback) -> glib::JoinHandle<()>
    where
        Callback: Fn(Message) -> F + 'static,
        F: Future<Output = ()>,
    {
        let mut receiver = self.subscribe();

        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(message) => f(message).await,
                    Err(e) => log::error!("{e}"),
                };
            }
        })
    }

    fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.sender.subscribe()
    }

    fn start_server(self) -> JoinHandle<()> {
        RT.spawn(async {
            let name = "org.freedesktop.Notifications";
            let path = "/org/freedesktop/Notifications";

            let connection = ConnectionBuilder::session()
                .expect("cant create session")
                .name(name)
                .expect("cant set name")
                .serve_at(path, self)
                .expect("cant serve")
                .build()
                .await;

            if let Err(e) = connection {
                log::error!("{e}");
            }

            future::pending::<()>().await;
        })
    }

    fn generate_id(&mut self, replaces_id: u32) -> u32 {
        if replaces_id == 0 {
            let id = self.notification_id;
            self.notification_id = id.wrapping_add(1);

            id
        } else {
            replaces_id
        }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn notify(&mut self, data: NotificationData) -> u32 {
        let id = self.generate_id(data.replaces_id);
        let message = Arc::new((id, data));

        if let Err(e) = self.sender.send(message) {
            log::error!("{e}");
        }

        id
    }

    fn get_server_information(&self) -> ServerInfo {
        ServerInfo {
            name: "Chameleon Notifications".to_string(),
            vendor: "decisivestrike".to_string(),
            version: "1.0".to_string(),
            spec_version: "1.2".to_string(),
        }
    }
}
