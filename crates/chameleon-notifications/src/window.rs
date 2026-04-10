use crate::GAP;
use crate::notification_data::NotificationData;
use crate::notification_list::NOTIFICATION_WINDOWS;
use grapes::gtk::Orientation;
use grapes::layer_shell::{Edge, LayerShell};
use grapes::prelude::{BoxExt, GtkWindowExt, WidgetExt};
use grapes::{WindowComponent, gtk::ApplicationWindow};
use grapes::{glib, gtk};

#[derive(WindowComponent)]
pub struct NotificationWindow {
    id: u32,

    #[root]
    pub window: ApplicationWindow,
}

impl NotificationWindow {
    pub fn new(app: &gtk::Application, data: &NotificationData) -> Self {
        let window = ApplicationWindow::new(app);
        Self::setup_layershell(&window);

        let root = Self::build_ui(&data);
        window.set_child(Some(&root));
        window.set_widget_name("notification-window");
        window.connect_destroy({
            let id = data.id;
            move |_| {
                glib::spawn_future_local(NOTIFICATION_WINDOWS.remove(id));
            }
        });

        glib::timeout_add_local_once(data.lifetime, {
            let title = data.title.clone();
            let window_clone = window.clone();

            move || {
                window_clone.destroy();
                log::debug!("notification destroyed: '{title}'");
            }
        });

        Self {
            id: data.id,
            window,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn update(&self, data: &NotificationData) {
        todo!("update logic")
    }

    fn setup_layershell(window: &ApplicationWindow) {
        window.init_layer_shell();
        window.set_namespace(Some("chameleon-notifications"));

        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, GAP);

        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Right, GAP);
    }

    fn build_ui(data: &NotificationData) -> gtk::Box {
        let title = gtk::Label::builder()
            .label(&data.title)
            .name("title")
            .build();

        let body = gtk::Label::builder().label(&data.body).name("body").build();

        let container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .name("notification")
            .build();

        container.append(&title);
        container.append(&body);

        container
    }
}
