mod card;
mod entry_object;

use chameleon_config::LauncherConfig;
use freedesktop_desktop_entry::desktop_entries;
use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    self, EventControllerKey, FilterChange, FilterListModel, ListItem,
    ListScrollFlags, ListView, PolicyType, ScrolledWindow,
    SignalListItemFactory, SingleSelection, SortListModel, SorterChange,
};
use grapes::layer_shell::{KeyboardMode, Layer, LayerShell};
use grapes::prelude::{
    BoxExt, Cast, CastNone, EditableExt, EntryExt, FilterExt, GtkWindowExt,
    ListItemExt, SorterExt, WidgetExt,
};
use grapes::tokio::process::Command;
use grapes::{RT, gio};
use grapes::{WindowComponent, gtk::ApplicationWindow};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env::home_dir;
use std::process::Stdio;
use std::rc::Rc;

use crate::card::Card;
use crate::entry_object::ApplicationEntry;

use nucleo::{Config, Matcher, Utf32Str};

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,
    entry: gtk::Entry,
    selection_model: SingleSelection,
    list_view: ListView,
    visibility: Cell<bool>,

    config: &'static LauncherConfig,
}

impl Launcher {
    pub fn create(
        application: &gtk::Application,
        config: &'static LauncherConfig,
    ) -> Rc<Self> {
        // get locale
        let locales = &["en".to_string()];
        let entries = Self::find_desktop_entries(locales);

        let launcher = Self::new(application, &entries, config);

        launcher.entry.connect_activate(clone!(
            #[weak]
            launcher,
            move |_| {
                let entry_info: ApplicationEntry = launcher
                    .selection_model
                    .selected_item()
                    .and_downcast()
                    .expect("cant cast");

                launcher.toggle_visibility();
                launcher.open(&entry_info.exec(), entry_info.terminal());
            }
        ));

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
        launcher.window.add_controller(controller);

        launcher
    }

    pub fn new(
        application: &gtk::Application,
        entries: &HashMap<String, ApplicationEntry>,
        config: &'static LauncherConfig,
    ) -> Rc<Self> {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let entry = gtk::Entry::builder()
            .placeholder_text(&*config.placeholder)
            .hexpand(true)
            .build();

        let (selection_model, list_view) =
            Self::setup_list_view(&entry, entries);

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .can_focus(true)
            .can_target(false)
            .min_content_width(360)
            .max_content_width(720)
            .min_content_height(400)
            .max_content_height(720)
            .overlay_scrolling(true)
            .child(&list_view)
            .build();

        let window = Self::create_application_window(application);

        container.append(&entry);
        container.append(&scrolled_window);

        window.set_child(Some(&container));

        Self {
            window,
            entry,
            selection_model,
            list_view,
            visibility: Cell::new(false),
            config,
        }
        .into()
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.visibility.get();

        if target_visibility {
            self.window.present();
        } else {
            self.window.set_visible(target_visibility);
            self.entry.set_text("");
            self.selection_model.set_selected(0);
            self.list_view.scroll_to(0, ListScrollFlags::FOCUS, None);
        }

        self.visibility.set(target_visibility);
    }

    fn find_desktop_entries(
        locales: &[String],
    ) -> HashMap<String, ApplicationEntry> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry)
                    .ok()
                    .map(|entry| (entry.name(), entry))
            })
            .collect()
    }

    fn setup_list_view(
        entry: &gtk::Entry,
        entries: &HashMap<String, ApplicationEntry>,
    ) -> (SingleSelection, ListView) {
        let model = gio::ListStore::new::<ApplicationEntry>();
        let values: Vec<_> = entries.values().map(Clone::clone).collect();
        model.extend_from_slice(&values);

        let selection_model = Self::setup_selection_model(entry, model);
        let item_factory = Self::create_factory();

        let list_view = ListView::builder()
            .model(&selection_model)
            .factory(&item_factory)
            .build();

        (selection_model, list_view)
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

    fn setup_selection_model(
        entry: &gtk::Entry,
        model: gio::ListStore,
    ) -> SingleSelection {
        let matcher = RefCell::new(Matcher::new(Config::DEFAULT));

        let filter = gtk::CustomFilter::new(clone!(
            #[strong]
            entry,
            move |obj| {
                let app_entry = obj
                    .downcast_ref::<ApplicationEntry>()
                    .expect("The object needs to be of type `EntryInfo`");

                let search_query = &entry.text().to_string().to_lowercase();
                let app_name = app_entry.name().to_lowercase();

                matcher
                    .borrow_mut()
                    .fuzzy_match(
                        Utf32Str::Ascii(app_name.as_bytes()),
                        Utf32Str::Ascii(search_query.as_bytes()),
                    )
                    .is_some()
            }
        ));
        let filter_model =
            FilterListModel::new(Some(model), Some(filter.clone()));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let entry_info_1 = obj1
                .downcast_ref::<ApplicationEntry>()
                .expect("The object needs to be of type `EntryInfo`");
            let entry_info_2 = obj2
                .downcast_ref::<ApplicationEntry>()
                .expect("The object needs to be of type `EntryInfo`");

            let str_1 = entry_info_1.name();
            let str_2 = entry_info_2.name();

            str_1.cmp(&str_2).into()
        });

        let filter_and_sort_model =
            SortListModel::new(Some(filter_model), Some(sorter.clone()));

        entry.connect_changed(clone!(
            #[strong]
            filter,
            #[strong]
            sorter,
            move |_| {
                filter.changed(FilterChange::Different);
                sorter.changed(SorterChange::Different);
            }
        ));

        SingleSelection::new(Some(filter_and_sort_model))
    }

    fn create_application_window(
        application: &gtk::Application,
    ) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);

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
        let config = self.config;

        RT.spawn(async move {
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
                        libc::setsid();
                        Ok(())
                    });
                }
            }

            match command.spawn() {
                Ok(_) => log::info!("App '{name}' spawned"),
                Err(e) => log::error!("Can't spawn '{name}'. Error: {e}"),
            }
        });
    }
}
