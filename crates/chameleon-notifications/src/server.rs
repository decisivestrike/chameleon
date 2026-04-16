//! https://specifications.freedesktop.org/notification/latest/protocol.html
use crate::requests::NotificationData;
use crate::responses::ServerInfo;
use grapes::RT;
use std::future::{self};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use zbus::connection::Builder as ConnectionBuilder;
use zbus::{Connection, interface};

pub struct NotificationServer {
    notification_id: u32,
    sender: mpsc::Sender<NotificationData>,
}

impl NotificationServer {
    const SERVICE_NAME: &str = "org.freedesktop.Notifications";
    const OBJECT_PATH: &str = "/org/freedesktop/Notifications";

    pub fn new() -> (Self, mpsc::Receiver<NotificationData>) {
        let (sender, receiver) = mpsc::channel(64);
        let server = Self {
            notification_id: 1,
            sender,
        };

        (server, receiver)
    }

    pub fn serve(self) -> JoinHandle<()> {
        RT.spawn(async {
            let connection = self.create_connection().await;

            if let Err(e) = connection {
                log::error!("{e}");
            }

            future::pending::<()>().await;
        })
    }

    async fn create_connection(self) -> zbus::Result<Connection> {
        ConnectionBuilder::session()?
            .name(Self::SERVICE_NAME)?
            .serve_at(Self::OBJECT_PATH, self)?
            .build()
            .await
    }

    fn generate_id(&mut self) -> u32 {
        let id = self.notification_id;
        self.notification_id = id.wrapping_add(1);

        id
    }

    /// If the id is zero, then we have to determine it ourselves
    fn set_id_if_zero(&mut self, data: &mut NotificationData) -> u32 {
        if data.replaces_id == 0 {
            let id = self.generate_id();
            data.replaces_id = id;

            id
        } else {
            data.replaces_id
        }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(&mut self, mut data: NotificationData) -> u32 {
        let id = self.set_id_if_zero(&mut data);

        log::debug!("{data:#?}");

        if let Err(e) = self.sender.send(data).await {
            log::error!("{e}");
        }

        id
    }

    fn get_server_information(&self) -> ServerInfo {
        ServerInfo::default()
    }
}
