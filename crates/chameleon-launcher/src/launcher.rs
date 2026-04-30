use crate::card::Card;
use crate::config::LauncherConfig;
use crate::entry_object::ApplicationEntry;
use freedesktop_desktop_entry::desktop_entries;
use gtk::gdk::Key;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{
    EventControllerKey, ListItem, ListScrollFlags, ListView, PolicyType,
    ScrolledWindow, SignalListItemFactory, SingleSelection, Window, gio, glib,
};
use gtke::WindowComponent;
use gtkio::future::spawn;
use layer_shell::{KeyboardMode, Layer, LayerShell};
use nucleo::{Config, Matcher, Utf32Str};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::env::home_dir;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{error, info};

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: Window,
    apps: Vec<ApplicationEntry>,
    searchbar: gtk::SearchEntry,
    selection_model: SingleSelection,
    list_view: ListView,
    list_store: gio::ListStore,
    visibility: Cell<bool>,

    config: LauncherConfig,
}

impl Launcher {
    pub fn create(config: LauncherConfig) -> Arc<Self> {
        let launcher = Self::new(config);

        // On enter hit
        launcher.searchbar.connect_activate(clone!(
            #[strong]
            launcher,
            move |_| {
                let maybe_entry = launcher
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
        launcher.window.add_controller(controller);

        // Sort + filter
        let matcher = RefCell::new(Matcher::new(Config::DEFAULT));

        let query = launcher.searchbar.text().to_string();
        launcher.filter_and_sort(&matcher, &query);

        launcher.searchbar.connect_changed(clone!(
            #[strong]
            launcher,
            move |searchbar| {
                let query = searchbar.text().to_string();
                launcher.filter_and_sort(&matcher, &query);
            }
        ));

        launcher
    }

    fn new(config: LauncherConfig) -> Arc<Self> {
        // get locale
        let locales = &["en".to_string()];
        let apps = Self::find_apps(locales);

        let searchbar = gtk::SearchEntry::builder()
            .placeholder_text(&*config.placeholder)
            .hexpand(true)
            .build();

        let list_store = gio::ListStore::new::<ApplicationEntry>();
        let selection_model = SingleSelection::new(Some(list_store.clone()));
        let item_factory = Self::create_factory();

        let list_view = ListView::builder()
            .model(&selection_model)
            .factory(&item_factory)
            .build();

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .can_focus(false)
            .can_target(false)
            .min_content_width(360)
            .max_content_width(720)
            .min_content_height(400)
            .max_content_height(720)
            .overlay_scrolling(true)
            .child(&list_view)
            .build();

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&searchbar);
        container.append(&scrolled_window);

        let window = Self::create_window();
        window.set_child(Some(&container));

        window.set_focusable(true);

        Self {
            window,
            apps,
            searchbar,
            selection_model,
            list_view,
            visibility: Cell::new(false),
            list_store,
            config,
        }
        .into()
    }

    fn filter_and_sort(&self, matcher: &RefCell<Matcher>, query: &String) {
        let mut updated_store: Vec<_> = self
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

        let len = self.list_store.n_items();
        self.list_store.splice(0, len, &updated_store);

        if updated_store.len() > 0 {
            self.list_view.scroll_to(0, ListScrollFlags::SELECT, None);
        }
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.visibility.get();

        if target_visibility {
            self.window.present();
        } else {
            self.window.set_visible(target_visibility);
            self.searchbar.set_text("");
            self.selection_model.set_selected(0);
            self.list_view.scroll_to(0, ListScrollFlags::FOCUS, None);
        }

        self.visibility.set(target_visibility);
    }

    fn find_apps(locales: &[String]) -> Vec<ApplicationEntry> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry).ok()
            })
            .collect()
    }

    fn create_factory() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        let on_setup = Self::create_factory_connect_setup_closure();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            on_setup(&list_item);
        });

        let on_bind = Self::create_factory_bind_closure();
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            on_bind(list_item);
        });

        factory
    }

    fn create_factory_connect_setup_closure() -> impl Fn(&ListItem) {
        |list_item: &ListItem| {
            let card = Card::new();

            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(card.as_ref()));
        }
    }

    fn create_factory_bind_closure() -> impl Fn(&ListItem) {
        |list_item: &ListItem| {
            let entry_info = list_item
                .item()
                .and_downcast::<ApplicationEntry>()
                .expect("The item has to be an EntryInfo");

            let container = list_item
                .child()
                .and_downcast::<gtk::Box>()
                .expect("The child has to be a Box");

            let card = Card::from(container);
            card.setup(&entry_info);
        }
    }

    fn create_window() -> gtk::Window {
        let window = gtk::Window::new();

        window.init_layer_shell();

        window.set_size_request(480, 240);
        window.set_widget_name("launcher");
        window.set_namespace(Some("chameleon-launcher"));
        window.set_exclusive_zone(-1);
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(if true {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::OnDemand
        });

        window
    }

    fn open(&self, name: &str, terminal: bool) {
        let name = name.to_string();
        let config = self.config.clone();

        spawn(async move {
            let mut command =
                if terminal && let Some(cmd) = &config.terminal_cmd {
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
        });
    }
}

unsafe impl Send for Launcher {}
unsafe impl Sync for Launcher {}
