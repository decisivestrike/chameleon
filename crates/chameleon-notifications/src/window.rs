use crate::notification_list::{GAP, NOTIFICATION_WINDOWS};
use crate::notifications::NotificationData;
use grapes::gtk::Orientation;
use grapes::gtk::ffi::GtkApplicationWindow;
use grapes::layer_shell::{Edge, LayerShell};
use grapes::prelude::{BoxExt, GtkWindowExt, ObjectType, WidgetExt};
use grapes::{WindowComponent, gtk::ApplicationWindow};
use grapes::{glib, gtk};

#[derive(WindowComponent)]
pub struct NotificationWindow {
    #[root]
    pub window: ApplicationWindow,
}

impl NotificationWindow {
    pub fn new(app: &gtk::Application, data: NotificationData) -> Self {
        let window = ApplicationWindow::new(app);

        window.init_layer_shell();

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, GAP);

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, GAP);

        let title = gtk::Label::new(Some(&data.title));
        let body = gtk::Label::new(Some(&data.body));

        let container = gtk::Box::new(Orientation::Vertical, 10);
        container.append(&title);
        container.append(&body);

        window.set_child(Some(&container));

        window.connect_destroy(move |window| {
            let ptr = window.as_ptr();
            glib::spawn_future_local(NOTIFICATION_WINDOWS.remove(ptr as usize));
        });

        glib::timeout_add_local_once(data.timeout, {
            let title = data.title.clone();
            let window_clone = window.clone();

            move || {
                window_clone.destroy();
                log::debug!("notification destroyed: '{title}'");
            }
        });

        Self { window }
    }

    pub fn as_ptr(&self) -> *mut GtkApplicationWindow {
        self.window.as_ptr()
    }
}
