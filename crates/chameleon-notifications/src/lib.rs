mod window;

mod notifications;

pub use notifications::Notifications;

mod server_information;
pub use server_information::ServerInformation;

use grapes::{
    WindowComponent, glib, gtk, tokio::sync::broadcast::error::RecvError,
};

use crate::window::NotificationWindow;

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
