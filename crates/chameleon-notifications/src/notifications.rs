use crate::ServerInformation;
use grapes::tokio::sync::broadcast::{self, Sender};
use std::collections::HashMap;
use std::time::Duration;
use zbus::connection::Builder as ConnectionBuilder;
use zbus::{Connection, interface};

static DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct NotificationData {
    pub title: String,
    pub body: String,
    pub timeout: Duration,
}

pub struct Notifications {
    sender: Sender<NotificationData>,
}

impl Notifications {
    pub fn new() -> Self {
        let sender = broadcast::Sender::new(64);

        Self { sender }
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
}

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn notify(
        &self,
        _app_name: &str,
        _replaces_id: u32,
        _app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<String>,
        _hints: HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let data = NotificationData {
            title: summary.to_string(),
            body: body.to_string(),
            timeout: match expire_timeout {
                timeout if timeout < 0 => DEFAULT_TIMEOUT,
                timeout => Duration::from_millis(timeout as u64),
            },
        };

        if let Err(e) = self.sender.send(data) {
            log::error!("{e}");
        }

        0
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
