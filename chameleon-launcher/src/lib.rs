use grapes::prelude::GtkWindowExt;
use grapes::{
    WindowComponent,
    gtk::{self, ApplicationWindow, gdk},
};

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,

    monitor: gdk::Monitor,
}

impl Launcher {
    pub fn new(application: &gtk::Application, monitor: &gdk::Monitor) -> Self {
        let window = gtk::ApplicationWindow::new(application);

        Self {
            window,
            monitor: monitor.clone(),
        }
    }
}
