pub mod modules;

use grapes::{
    extensions::GrapesBoxExt,
    gtk::{
        self, ApplicationWindow, Orientation,
        gdk::{self, prelude::MonitorExt},
        prelude::{GtkWindowExt, WidgetExt},
    },
    layer_shell::{Edge, KeyboardMode, Layer, LayerShell},
};
use serde::Deserialize;

pub enum Position {
    Top,
    Right,
    Bottom,
    Left,
}

pub enum Placement {
    Start,
    Center,
    End,
}

pub struct Bar {
    window: ApplicationWindow,
    modules_start: gtk::Box,
    modules_center: gtk::Box,
    modules_end: gtk::Box,
}

impl Bar {
    pub fn new(
        application: &gtk::Application,
        monitor: &gdk::Monitor,
        thickness: Option<i32>,
        spacing: i32,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        let cb = gtk::CenterBox::new();
        window.set_child(Some(&cb));

        let modules_start = gtk::Box::new(Orientation::Horizontal, spacing);
        let modules_center = gtk::Box::new(Orientation::Horizontal, spacing);
        let modules_end = gtk::Box::new(Orientation::Horizontal, spacing);

        cb.set_start_widget(Some(&modules_start));
        cb.set_center_widget(Some(&modules_center));
        cb.set_end_widget(Some(&modules_end));

        let bar = Self {
            window,
            modules_start,
            modules_center,
            modules_end,
        };

        bar.setup(monitor, thickness);

        bar
    }

    pub fn add_module(&self, module: impl AsRef<gtk::Widget>, placement: Placement) {
        match placement {
            Placement::Start => self.modules_start.append_ref(module),
            Placement::Center => self.modules_center.append_ref(module),
            Placement::End => self.modules_end.append_ref(module),
        }
    }

    /// This is almost an ordinary window, so it should be presented.
    pub fn present(&self) {
        self.window.present();
    }

    fn setup(&self, monitor: &gdk::Monitor, thickness: Option<i32>) {
        let window = &self.window;

        window.init_layer_shell();

        window.set_widget_name("bar");
        window.set_default_width(monitor.geometry().width());

        window.set_decorated(false);
        window.set_resizable(false);

        window.set_keyboard_mode(KeyboardMode::OnDemand);

        if let Some(thickness) = thickness {
            window.set_default_height(thickness);
            window.set_exclusive_zone(thickness);
        } else {
            window.auto_exclusive_zone_enable();
        }

        window.set_layer(Layer::Top);

        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        // window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);

        window.set_monitor(Some(monitor));
    }
}

#[derive(Debug, Deserialize)]
pub struct BarConfig {
    #[serde(default)]
    pub enabled: bool,
    pub thickness: Option<i32>,
    #[serde(default)]
    pub spacing: i32,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,
}
