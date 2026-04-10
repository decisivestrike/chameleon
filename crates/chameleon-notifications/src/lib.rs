pub mod notification_list;
pub mod window;

mod notifications;
pub use notifications::Notifications;

mod server_information;
pub use server_information::ServerInformation;

use crate::{
    notification_list::NOTIFICATION_WINDOWS, window::NotificationWindow,
};
use grapes::{
    WindowComponent, glib, gtk, tokio::sync::broadcast::error::RecvError,
};

pub fn setup(app: &gtk::Application) -> Notifications {
    let notifications = Notifications::new();
    let mut receiver = notifications.subscribe();

    let app_clone = app.clone();
    glib::spawn_future_local(async move {
        loop {
            match receiver.recv().await {
                Ok(data) => {
                    let window = NotificationWindow::new(&app_clone, data);
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
