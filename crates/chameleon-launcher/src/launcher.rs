use crate::config::LauncherConfig;
use crate::entry_object::ApplicationEntry;
use freedesktop_desktop_entry::desktop_entries;
use gtk::gdk::Key;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{Object, clone};
use gtk::prelude::*;
use gtk::{
    EventControllerKey, ListScrollFlags, ListView, PolicyType, ScrolledWindow,
    SingleSelection, gio, glib,
};
use layer_shell::{KeyboardMode, Layer, LayerShell};
use nucleo::{Config, Matcher, Utf32Str};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::env::home_dir;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tracing::{error, info};

mod imp {
    use super::*;
    use crate::factory::Factory;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::OnceCell;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::Launcher)]
    pub struct LauncherImp {
        pub apps: Vec<ApplicationEntry>,
        pub searchbar: gtk::Entry,
        pub selection_model: SingleSelection,
        pub list_view: ListView,
        pub list_store: gio::ListStore,
        pub config: OnceCell<LauncherConfig>,
    }

    impl Default for LauncherImp {
        fn default() -> Self {
            let list_store = gio::ListStore::new::<ApplicationEntry>();
            let selection_model =
                SingleSelection::new(Some(list_store.clone()));

            let list_view = ListView::builder()
                .model(&selection_model)
                .factory(&Factory::new())
                .build();

            Self {
                apps: super::Launcher::find_apps(&["en".to_string()]),
                searchbar: Default::default(),
                config: OnceCell::new(),
                selection_model,
                list_view,
                list_store,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherImp {
        const NAME: &'static str = "ChameleonLauncher";
        type Type = super::Launcher;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for LauncherImp {
        fn constructed(&self) {
            self.parent_constructed();

            let scrolled_window = ScrolledWindow::builder()
                .propagate_natural_height(true)
                .valign(gtk::Align::Start)
                .hscrollbar_policy(PolicyType::Never)
                .vscrollbar_policy(PolicyType::Automatic)
                .can_focus(false)
                .can_target(false)
                .min_content_width(480)
                .max_content_width(720)
                .min_content_height(0)
                .max_content_height(420)
                .hexpand(false)
                .vexpand(false)
                .overlay_scrolling(true)
                .child(&self.list_view)
                .build();

            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .valign(gtk::Align::Start)
                .hexpand(false)
                .vexpand(false)
                .homogeneous(false)
                .spacing(0)
                .build();

            container.append(&self.searchbar);
            container.append(&scrolled_window);

            let window = self.obj();
            window.init_layer_shell();
            window.set_default_size(-1, -1); // Auto size
            window.set_vexpand(false);
            window.set_hexpand(false);
            window.set_widget_name("launcher");
            window.set_namespace(Some("chameleon-launcher"));
            window.set_layer(Layer::Top);
            window.set_keyboard_mode(if true {
                KeyboardMode::Exclusive
            } else {
                KeyboardMode::OnDemand
            });

            window.set_child(Some(&container));
            window.set_focusable(true);
        }
    }

    impl WidgetImpl for LauncherImp {}

    impl WindowImpl for LauncherImp {}
}

glib::wrapper! {
    pub struct Launcher(ObjectSubclass<imp::LauncherImp>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Launcher {
    pub fn new(config: LauncherConfig) -> Self {
        let launcher: Self = Object::builder().build();

        let imp = launcher.imp();
        imp.searchbar
            .set_placeholder_text(Some(&*config.placeholder));
        imp.config.set(config).unwrap();

        // On enter hit
        imp.searchbar.connect_activate(clone!(
            #[strong]
            launcher,
            move |_| {
                let maybe_entry = launcher
                    .imp()
                    .selection_model
                    .selected_item()
                    .and_downcast::<ApplicationEntry>();

                if let Some(entry) = maybe_entry {
                    launcher.toggle_visibility();
                    launcher.open(&entry.exec(), entry.terminal());
                }
            }
        ));

        // Exit on esc
        let controller = EventControllerKey::new();
        controller.connect_key_pressed(clone!(
            #[strong]
            launcher,
            move |_, key, _, _| {
                if key == Key::Escape {
                    launcher.toggle_visibility();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        launcher.add_controller(controller);

        // Sort + filter
        let matcher = RefCell::new(Matcher::new(Config::DEFAULT));
        let query = imp.searchbar.text().to_string();
        launcher.filter_and_sort(&matcher, &query);

        imp.searchbar.connect_changed(clone!(
            #[strong]
            launcher,
            move |searchbar| {
                let query = searchbar.text().to_string();
                launcher.filter_and_sort(&matcher, &query);
            }
        ));

        launcher
    }

    fn filter_and_sort(&self, matcher: &RefCell<Matcher>, query: &String) {
        let mut updated_store: Vec<_> = self
            .imp()
            .apps
            .iter()
            .filter_map(|app| {
                let score = matcher.borrow_mut().fuzzy_match(
                    Utf32Str::Ascii(app.name().as_bytes()),
                    Utf32Str::Ascii(query.as_bytes()),
                )?;

                Some((app.clone(), score))
            })
            .collect();

        updated_store.sort_by(|first, second| {
            let first_score = first.1;
            let second_score = second.1;

            let score_cmp = second_score.cmp(&first_score);

            let first_name = first.0.name();
            let second_name = second.0.name();

            match score_cmp {
                Ordering::Equal => first_name.cmp(&second_name),
                _ => score_cmp,
            }
        });

        let updated_store: Vec<_> =
            updated_store.into_iter().map(|e| e.0).collect();

        let len = self.imp().list_store.n_items();
        self.imp().list_store.splice(0, len, &updated_store);

        let model = self.imp().list_view.model().unwrap();
        if model.n_items() > 0 {
            self.imp()
                .list_view
                .scroll_to(0, ListScrollFlags::SELECT, None);
        }
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.get_visible();

        if target_visibility {
            self.present();
        } else {
            self.set_visible(target_visibility);

            let imp = self.imp();
            imp.searchbar.set_text("");
            imp.selection_model.set_selected(0);
            imp.list_view.scroll_to(0, ListScrollFlags::FOCUS, None);
        }
    }

    fn find_apps(locales: &[String]) -> Vec<ApplicationEntry> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry).ok()
            })
            .collect()
    }

    fn open(&self, name: &str, terminal: bool) {
        let name = name.to_string();
        let config = self.imp().config.get().unwrap();

        let mut command = if terminal && let Some(cmd) = &config.terminal_cmd {
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

        if config.detach {
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
}
