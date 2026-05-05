//! https://specifications.freedesktop.org/notification/latest/protocol.html
pub mod info;
pub use info::{Capability, ServerInfo};

use crate::notification::NotificationData;
use gtkio::future::spawn;
use std::future::{self};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use zbus::connection::Builder as ConnectionBuilder;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";

pub struct NotificationServer {
    notification_id: u32,
    sender: mpsc::Sender<NotificationData>,
}

impl NotificationServer {
    const SERVICE_NAME: &str = "org.freedesktop.Notifications";
    const OBJECT_PATH: &str = "/org/freedesktop/Notifications";

    pub fn create() -> (Self, mpsc::Receiver<NotificationData>) {
        let (sender, receiver) = mpsc::channel(64);
        let server = Self::new(sender);

        (server, receiver)
    }

    pub fn serve(self) -> JoinHandle<()> {
        spawn(async {
            let connection = self.create_connection().await;

            if let Err(e) = connection {
                error!("{e}");
            }

            future::pending::<()>().await;
        })
    }

    fn new(sender: mpsc::Sender<NotificationData>) -> Self {
        Self {
            notification_id: 1,
            sender,
        }
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
        if data.id == 0 {
            let id = self.generate_id();
            data.id = id;

            id
        } else {
            data.id
        }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    /// Returns the server's capabilities.
    async fn get_capabilities(&self) -> Vec<Capability> {
        vec![Capability::Body, Capability::BodyMarkup]
    }

    async fn notify(&mut self, mut data: NotificationData) -> u32 {
        let id = self.set_id_if_zero(&mut data);

        debug!("{data:#?}");

        if let Err(e) = self.sender.send(data).await {
            error!("{e}");
        }

        id
    }

    async fn close_notification(&self, _id: u32) {
        warn!("I can't close notifications yet.");
    }

    fn get_server_information(&self) -> ServerInfo {
        ServerInfo::default()
    }

    #[zbus(signal)]
    async fn notification_closed(
        signal_emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn activation_token(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}
