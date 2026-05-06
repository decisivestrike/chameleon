//! https://specifications.freedesktop.org/notification/latest/protocol.html
pub mod info;
pub use info::{Capability, ServerInfo};

pub mod action;
pub use action::Action;

use crate::notification::NotificationData;
use gtkio::future::spawn;
use std::future;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};
use zbus::connection::Builder as ConnectionBuilder;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};

/// The specification version the server is compliant with.
pub const SPECIFICATION_VERSION: &str = "1.2";

pub struct NotificationServer {
    notification_id: u32,
    sender: mpsc::Sender<Action>,
}

impl NotificationServer {
    const SERVICE_NAME: &str = "org.freedesktop.Notifications";
    const OBJECT_PATH: &str = "/org/freedesktop/Notifications";

    pub fn create() -> (Self, mpsc::Receiver<Action>) {
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

    fn new(sender: mpsc::Sender<Action>) -> Self {
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
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    /// Returns the server's capabilities.
    fn get_capabilities(&self) -> Vec<Capability> {
        trace!("GetCapabilities");
        vec![Capability::Body, Capability::BodyMarkup]
    }

    async fn notify(&mut self, mut data: NotificationData) -> u32 {
        trace!("Notify");

        let id = {
            if data.id == 0 {
                data.id = self.generate_id();
            }

            data.id
        };

        debug!("{data:#?}");

        if let Err(e) = self.sender.send(Action::Create(data)).await {
            error!("{e}");
        }

        id
    }

    async fn close_notification(&self, id: u32) {
        trace!("CloseNotification({})", id);

        if let Err(e) = self.sender.send(Action::Remove(id)).await {
            error!("{e}");
        }
    }

    fn get_server_information(&self) -> ServerInfo {
        trace!("GetServerInformation");
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
