use gtk::glib;
use notifications::manager::NotificationManager;
use std::process::exit;
use tracing::error;

fn main() {
    tracing_subscriber::fmt().without_time().init();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let manager = NotificationManager::new();
    manager.run();

    glib::MainLoop::new(None, false).run();
}
