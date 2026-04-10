pub mod notification_data;
pub mod notification_list;
pub mod window;

mod notifications_server;
use std::time::Duration;

pub use notifications_server::NotificationsServer;

mod server_information;
pub use server_information::ServerInformation;

use crate::{
    notification_list::NOTIFICATION_WINDOWS, window::NotificationWindow,
};
use grapes::{
    WindowComponent, glib, gtk, tokio::sync::broadcast::error::RecvError,
};

pub static DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);
pub static MAX_NOTIFICATIONS: usize = 5;
pub static GAP: i32 = 10;

pub fn setup(app: &gtk::Application) -> NotificationsServer {
    let notifications = NotificationsServer::new();
    let mut receiver = notifications.subscribe();

    let app_clone = app.clone();
    glib::spawn_future_local(async move {
        loop {
            match receiver.recv().await {
                Ok(data) => {
                    let window = NotificationWindow::new(&app_clone, &data);
                    window.present();

                    NOTIFICATION_WINDOWS.append(window).await;
                }
                Err(e) => {
                    log::error!("{e}");

                    if let RecvError::Closed = e {
                        std::process::exit(-1);
                    }
                }
            }
        }
    });

    notifications
}
