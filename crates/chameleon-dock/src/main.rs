use std::collections::HashSet;
use std::fs;

use chameleon_shared::css::{Css, StylePriority};
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use layer_shell::{Edge, Layer, LayerShell};

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct Dock {
        pub apps_box: RefCell<gtk::Box>, // горизонтальный контейнер для иконок
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Dock {
        const NAME: &'static str = "MyDock";
        type Type = super::Dock;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for Dock {
        fn constructed(&self) {
            self.parent_constructed();
            let win = self.obj();

            // Настройка окна как панели в нижней части экрана
            win.set_decorated(false);
            win.set_resizable(false);
            win.set_widget_name("dock-window");

            // Layer shell
            win.init_layer_shell();
            win.set_layer(Layer::Bottom);
            win.set_anchor(Edge::Bottom, true);
            win.set_margin(Edge::Bottom, 40);
            // win.set_anchor(Edge::Left, true);
            // win.set_anchor(Edge::Right, true);
            // Минимальная высота (будет подстраиваться под содержимое)
            // win.set_default_size(1, 60);

            // Основной контейнер для иконок
            let apps_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            apps_box.set_halign(gtk::Align::Center);
            apps_box.set_valign(gtk::Align::Center);
            apps_box.set_widget_name("dock");

            win.set_child(Some(&apps_box));
            self.apps_box.replace(apps_box);
        }
    }

    impl WidgetImpl for Dock {}
    impl WindowImpl for Dock {}
}

glib::wrapper! {
    pub struct Dock(ObjectSubclass<imp::Dock>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Dock {
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Загружает список desktop-приложений и добавляет их иконки в док
    pub fn populate_apps(&self) {
        let imp = self.imp();
        let apps_box = imp.apps_box.borrow();

        // Получаем список всех desktop-приложений
        let app_infos = gio::AppInfo::all();

        let f = fs::read_to_string("/tmp/chameleon/dockapps").unwrap();
        let pinned_app_names: HashSet<&str> = f.split('\n').collect();

        for info in app_infos.iter() {
            println!("{}", info.name().as_str());

            if let Some(icon) = info.icon()
                && pinned_app_names.contains(info.name().as_str())
            {
                let image = gtk::Image::from_gicon(&icon);
                image.set_pixel_size(64);
                image.add_css_class("dock-icon");

                apps_box.append(&image);
            }
        }
    }
}

fn main() {
    gtk::init().unwrap();

    let dock = Dock::new();

    Css::load("/home/inqlog/.config/chameleon/styles.css")
        .apply(StylePriority::User);

    dock.populate_apps();
    dock.present();

    glib::MainLoop::new(None, false).run();
}
