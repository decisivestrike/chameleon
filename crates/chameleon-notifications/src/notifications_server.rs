use crate::ServerInformation;
use crate::notification_data::NotificationData;
use grapes::tokio::sync::broadcast;
use std::collections::HashMap;
use zbus::connection::Builder as ConnectionBuilder;
use zbus::{Connection, interface};

pub struct NotificationsServer {
    notification_id: u32,
    sender: broadcast::Sender<NotificationData>,
}

impl NotificationsServer {
    pub fn new() -> Self {
        Self {
            notification_id: 1,
            sender: broadcast::Sender::new(64),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NotificationData> {
        self.sender.subscribe()
    }

    pub async fn connection(self) -> Result<Connection, zbus::Error> {
        ConnectionBuilder::session()?
            .name("org.freedesktop.Notifications")?
            .serve_at("/org/freedesktop/Notifications", self)?
            .build()
            .await
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
impl NotificationsServer {
    fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let id = self.generate_id(replaces_id);

        let data = NotificationData::new(
            id,
            app_name,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        );

        if let Err(e) = self.sender.send(data) {
            log::error!("{e}");
        }

        id
    }

    fn get_server_information(&self) -> ServerInformation {
        ServerInformation {
            name: "Chameleon Notifications".to_string(),
            vendor: "decisivestrike".to_string(),
            version: "1.0".to_string(),
            spec_version: "1.2".to_string(),
        }
    }
}
