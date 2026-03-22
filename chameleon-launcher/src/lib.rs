mod card;
mod entry_object;

use freedesktop_desktop_entry::desktop_entries;
use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    self, EventControllerKey, FilterChange, FilterListModel, ListItem,
    ListScrollFlags, ListView, Orientation, PolicyType, ScrolledWindow,
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

use std::cell::Cell;
use std::collections::HashMap;
use std::env::home_dir;
use std::process::Stdio;
use std::rc::Rc;

use crate::card::Card;
use crate::entry_object::EntryInfo;

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,
    entry: gtk::Entry,
    selection_model: SingleSelection,
    list_view: ListView,
    visibility: Cell<bool>,
}

impl Launcher {
    pub fn create(application: &gtk::Application) -> Rc<Self> {
        // get locale
        let locales = &["en".to_string()];
        let entries = Self::find_desktop_entries(locales);

        let launcher = Self::new(application, &entries);

        launcher.entry.connect_activate(clone!(
            #[weak]
            launcher,
            move |_| {
                let entry_info: EntryInfo = launcher
                    .selection_model
                    .selected_item()
                    .and_downcast()
                    .expect("cant cast");

                launcher.toggle_visibility();
                launcher.open(&entry_info.exec());
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
        entries: &HashMap<String, EntryInfo>,
    ) -> Rc<Self> {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let entry = gtk::Entry::builder()
            .placeholder_text("Explore...")
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

    fn find_desktop_entries(locales: &[String]) -> HashMap<String, EntryInfo> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|entry| {
                let group = entry.groups.desktop_entry()?;

                match group.entry("NoDisplay") {
                    None | Some("false") => (),
                    _ => return None,
                };

                match group.entry("Hidden") {
                    None | Some("false") => (),
                    _ => return None,
                };

                let name = group.entry("Name")?.to_string();

                let exec =
                    Self::remove_field_codes(group.entry("Exec")?.to_string());
                let comment = group
                    .entry("Comment")
                    .map(String::from)
                    .unwrap_or(String::new());
                let icon = group
                    .entry("Icon")
                    .map(String::from)
                    .unwrap_or(String::new());

                let entry_info =
                    EntryInfo::new(name.clone(), exec, comment, icon);

                Some((name, entry_info))
            })
            .collect()
    }

    fn setup_list_view(
        entry: &gtk::Entry,
        entries: &HashMap<String, EntryInfo>,
    ) -> (SingleSelection, ListView) {
        let model = gio::ListStore::new::<EntryInfo>();
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
                .and_downcast::<EntryInfo>()
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
        let filter = gtk::CustomFilter::new(clone!(
            #[strong]
            entry,
            move |obj| {
                let entry_info = obj
                    .downcast_ref::<EntryInfo>()
                    .expect("The object needs to be of type `EntryInfo`");

                entry_info
                    .name()
                    .to_lowercase()
                    .contains(&entry.text().to_string().to_lowercase())
            }
        ));
        let filter_model =
            FilterListModel::new(Some(model), Some(filter.clone()));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let entry_info_1 = obj1
                .downcast_ref::<EntryInfo>()
                .expect("The object needs to be of type `EntryInfo`");
            let entry_info_2 = obj2
                .downcast_ref::<EntryInfo>()
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

    /// Removes field codes from a desktop entry `Exec` line.
    ///
    /// For more details, see
    /// [here](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html)
    fn remove_field_codes(mut exec: String) -> String {
        const FIELD_CODES: &[&str] = &[
            "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%v", "%m", "%i",
            "%c", "%k",
        ];

        for code in FIELD_CODES {
            exec = exec.replace(code, "");
        }

        exec.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn open(&self, name: &str) {
        let name = name.to_string();

        RT.spawn(async move {
            let mut base_command = Command::new("sh");
            let command = base_command
                .arg("-c")
                .arg(&name)
                .current_dir(home_dir().expect("can get $HOME"))
                .env_remove("RUST_LOG")
                .env_remove("RUST_BACKTRACE")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());

            let detach = true;

            if detach {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_field_codes() {
        assert_eq!(
            Launcher::remove_field_codes("firefox %u".to_string()),
            "firefox"
        );
        assert_eq!(
            Launcher::remove_field_codes("zeditor %U".to_string()),
            "zeditor"
        );
        assert_eq!(
            Launcher::remove_field_codes("myapp %f %F %u %U".to_string()),
            "myapp"
        );
        assert_eq!(
            Launcher::remove_field_codes("app %f arg1 %U arg2".to_string()),
            "app arg1 arg2"
        );
    }
}
