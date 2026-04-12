//! https://specifications.freedesktop.org/notification/latest/protocol.html
pub mod command;
pub use command::NotificationCommand;

use crate::{requests::NotificationData, responses::ServerInfo};
use grapes::RT;
use std::future::{self};
use tokio::{sync::mpsc, task::JoinHandle};
use zbus::{connection::Builder as ConnectionBuilder, interface};

pub struct NotificationServer {
    current_notification_id: u32,
    sender: mpsc::Sender<NotificationCommand>,
}

impl NotificationServer {
    pub fn new() -> (Self, mpsc::Receiver<NotificationCommand>) {
        let (sender, receiver) = mpsc::channel(64);

        (
            Self {
                current_notification_id: 1,
                sender,
            },
            receiver,
        )
    }

    pub fn serve(self) -> JoinHandle<()> {
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

    fn define_id(&mut self, replaces_id: u32) -> (u32, bool) {
        if replaces_id == 0 {
            let id = self.current_notification_id;
            self.current_notification_id = id.wrapping_add(1);

            (id, false)
        } else {
            (replaces_id, true)
        }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(&mut self, data: NotificationData) -> u32 {
        let (id, replace) = self.define_id(data.replaces_id);
        let command = NotificationCommand::new(id, data, replace);

        if let Err(e) = self.sender.send(command).await {
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
