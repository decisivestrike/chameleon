use grapes::glib::{self, clone};
use grapes::gtk::EventControllerKey;
use grapes::gtk::gdk::Key;
use grapes::layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use grapes::prelude::{GtkWindowExt, WidgetExt};
use grapes::{
    WindowComponent,
    gtk::{self, ApplicationWindow},
};

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,
}

impl Launcher {
    pub fn new(application: &gtk::Application) -> Self {
        let window = Self::create_configured_application_window(application);

        Self { window }
    }

    pub fn toggle_visibility(&self) {
        let current_visibility = self.window.is_visible();
        self.window.set_visible(!current_visibility);

        // if !current_visibility {
        //     self.window.grab_focus();
        // }
    }

    fn create_configured_application_window(
        application: &gtk::Application,
    ) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);
        window.init_layer_shell();

        window.set_widget_name("launcher");

        let label = gtk::Label::new(Some("IT WORKS"));
        window.set_child(Some(&label));

        window.set_namespace(Some("chameleon-launcher"));
        // window.set_exclusive_zone(-1);
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(if true {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::OnDemand
        });

        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);

        let controller = EventControllerKey::new();
        controller.connect_key_pressed(clone!(
            #[strong]
            window,
            move |_, key, _, _| {
                if key == Key::Escape {
                    window.set_visible(false);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        window.add_controller(controller);

        window.present();
        window.set_visible(false);

        window
    }
}
