mod cli;
mod config;
mod entry;

use crate::cli::Args;
use crate::config::Config;
use crate::entry::ApplicationEntry;
use chameleon_shared::css::{Css, StylePriority};
use chameleon_shared::styles_watcher;
use chameleon_shared::utils::read_config;
use freedesktop_desktop_entry::desktop_entries;
use gtk::glib;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtkio::future::spawn;
use layer_shell::{Edge, Layer, LayerShell};
use std::collections::HashSet;
use std::env::home_dir;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio, exit};
use tracing::{debug, error, info};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct Dock {
        pub apps_box: RefCell<gtk::Box>,
        pub apps: RefCell<HashSet<String>>,
        pub terminal_cmd: RefCell<Option<String>>,
        pub detach: Cell<bool>,
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

            win.set_decorated(false);
            win.set_resizable(false);
            win.set_widget_name("dock-window");

            win.init_layer_shell();
            win.set_namespace(Some("chameleon-dock"));
            win.set_layer(Layer::Bottom);
            win.set_anchor(Edge::Bottom, true);
            win.set_margin(Edge::Bottom, 40);

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
    pub fn new(config: Config) -> Self {
        let dock: Self = Object::builder().build();
        let Config {
            apps,
            terminal_cmd,
            detach,
        } = config;

        dock.imp().apps.replace(apps);
        dock.imp().terminal_cmd.replace(terminal_cmd);
        dock.imp().detach.replace(detach);

        dock
    }

    fn find_apps() -> Vec<ApplicationEntry> {
        desktop_entries(&["en".to_string(), "ru".to_string()])
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry).ok()
            })
            .collect()
    }

    pub fn populate_apps(&self) {
        let imp = self.imp();
        let apps_box = imp.apps_box.borrow();
        let pinned_app_names = self.imp().apps.borrow();

        for info in Self::find_apps().into_iter() {
            if pinned_app_names.contains(info.name().as_str()) {
                let image = gtk::Image::from_icon_name(&info.icon());
                image.set_pixel_size(64);
                image.add_css_class("dock-icon");

                let controller = gtk::GestureClick::new();
                let name = info.exec();
                let is_terminal = info.terminal();

                controller.connect_pressed(clone!(
                    #[strong(rename_to=dock)]
                    self,
                    move |_, _, _, _| {
                        dock.open_app(&name, is_terminal);
                    }
                ));

                image.add_controller(controller);
                apps_box.append(&image);
            }
        }
    }

    fn open_app(&self, name: &String, is_terminal: bool) {
        let mut command = if is_terminal
            && let Some(cmd) = self.imp().terminal_cmd.borrow().as_ref()
        {
            let mut command = Command::new(&cmd);
            command.arg(&name);

            command
        } else {
            let mut command = Command::new("sh");
            command.arg("-c").arg(&name);

            command
        };

        let command = command
            .current_dir(home_dir().expect("can get $HOME"))
            .env_remove("RUST_LOG")
            .env_remove("RUST_BACKTRACE")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if self.imp().detach.get() {
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        error!("setsid")
                    }

                    Ok(())
                });
            }
        }

        match command.spawn() {
            Ok(_) => info!("App '{name}' spawned"),
            Err(e) => error!("Can't spawn '{name}'. Error: {e}"),
        }
    }

    pub fn run(&self) {
        self.populate_apps();
        self.present();
    }
}

fn main() {
    gtk::init().unwrap();

    if let Err(e) = gtk::init() {
        error!("Не удалось инициализировать GTK: {e}");
        exit(1);
    };

    let Args {
        styles_path,
        config_path,
    } = argh::from_env();

    let config: Config = match read_config(&config_path) {
        Ok(config) => {
            debug!("{:#?}", config);
            config
        }
        Err(e) => {
            error!("{e}");
            exit(1)
        }
    };

    Css::load(&styles_path).apply(StylePriority::User);

    let dock = Dock::new(config);
    spawn(styles_watcher(styles_path.into()));

    dock.run();

    glib::MainLoop::new(None, false).run();
}
