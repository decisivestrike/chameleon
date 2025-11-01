use grapes::{
    gtk::{
        self, ApplicationWindow,
        gdk::{self, prelude::MonitorExt},
        prelude::{GtkWindowExt, WidgetExt},
    },
    layer_shell::{Edge, KeyboardMode, Layer, LayerShell},
};

pub enum BarPosition {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Bar {
    window: ApplicationWindow,
    cb: gtk::CenterBox,
}

impl Bar {
    pub fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        thickness: Option<u16>,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        let cb = gtk::CenterBox::new();

        let bar = Self { window, cb };

        bar.setup(monitor, thickness);

        bar
    }

    /// This is almost an ordinary window, so it should be presented.
    pub fn present(&self) {
        self.window.present();
    }

    fn setup(&self, monitor: &gdk::Monitor, thickness: Option<u16>) {
        let window = &self.window;

        window.set_widget_name("bar");
        window.set_default_size(monitor.geometry().width(), 30);

        window.set_decorated(false);
        window.set_resizable(false);

        window.init_layer_shell();
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Top);

        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        // window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);

        window.set_monitor(Some(monitor));
    }
}
