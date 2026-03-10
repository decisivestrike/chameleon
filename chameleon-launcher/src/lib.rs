use std::collections::BTreeSet;
use std::env::var;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use freedesktop_desktop_entry::{DesktopEntry, desktop_entries};
use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    EventControllerKey, FilterChange, FilterListModel, GridView, Label,
    ListItem, ListView, PolicyType, ScrolledWindow, SelectionModel,
    SignalListItemFactory, SingleSelection, SortListModel, SorterChange,
    StringList, Widget,
};
use grapes::layer_shell::{KeyboardMode, Layer, LayerShell};
use grapes::prelude::{
    BoxExt, Cast, EditableExt, EntryExt, FilterExt,
    GObjectPropertyExpressionExt, GtkWindowExt, IsA, ListItemExt, ListModelExt,
    SorterExt, WidgetExt,
};
use grapes::{
    WindowComponent,
    gtk::{self, ApplicationWindow},
};
use gtk::StringObject;

enum View {
    Grid(GridView),
    List(ListView),
}

enum Mode {
    Run,
    Drun,
}

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,

    desktop_entries: Vec<DesktopEntry>,
    list_executables: StringList,
    container: gtk::Box,
    entry: gtk::Entry,
    list_view: ListView,
}

impl Launcher {
    pub fn new(application: &gtk::Application) -> Self {
        // get locale
        let desktop_entries = desktop_entries(&["ru".to_string()])
            .into_iter()
            .filter_map(|entry| {
                let group = entry.groups.desktop_entry()?;
                group.entry("Exec")?;

                match group.entry("NoDisplay") {
                    None | Some("false") => (),
                    _ => return None,
                };

                match group.entry("Hidden") {
                    None | Some("false") => (),
                    _ => return None,
                };

                Some(entry)
            })
            .collect();

        let list_executables = list_executables_from_path();

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let entry = gtk::Entry::new();
        entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            println!("Enter нажат: {}", text);

            entry.set_text("");
        });
        container.append(&entry);

        let model = Self::setup_drun_model(&entry, &desktop_entries);
        let factory = Self::create_factory();

        let list_view = ListView::new(Some(model.clone()), Some(factory));
        list_view.connect_activate(move |list, i| {
            let item = list.model().and_then(|m| m.item(i));
            println!("Item: {item:?}");
        });

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .min_content_width(360)
            .min_content_height(400)
            .overlay_scrolling(true)
            .child(&list_view)
            .build();

        container.append(&scrolled_window);

        let window = Self::create_configured_application_window(application);
        window.set_child(Some(&container));

        Self {
            desktop_entries,
            list_executables,
            window,
            container,
            entry,
            list_view,
        }
    }

    pub fn toggle_visibility(&self) {
        let current_visibility = self.window.is_visible();
        self.window.set_visible(!current_visibility);

        // if !current_visibility {
        //     self.window.grab_focus();
        // }
    }

    fn application_list(desktop_entries: &Vec<DesktopEntry>) -> StringList {
        desktop_entries
            .iter()
            .filter_map(|entry| {
                let group = entry.groups.desktop_entry().unwrap();
                Some(group.entry("Name")?.to_string())
            })
            .collect()
    }

    fn create_factory() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let label = Label::new(None);
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");
            list_item.set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<StringObject>("string")
                .bind(&label, "label", Widget::NONE);
        });

        factory
    }

    fn setup_model_base(
        entry: &gtk::Entry,
        model: StringList,
    ) -> SingleSelection {
        let filter = gtk::CustomFilter::new(clone!(
            #[strong]
            entry,
            move |obj| {
                let string_object = obj
                    .downcast_ref::<StringObject>()
                    .expect("The object needs to be of type `StringObject`");

                string_object
                    .string()
                    .to_lowercase()
                    .contains(&entry.text().to_string().to_lowercase())
            }
        ));
        let filter_model =
            FilterListModel::new(Some(model), Some(filter.clone()));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let string_object_1 = obj1
                .downcast_ref::<StringObject>()
                .expect("The object needs to be of type `StringObject`");
            let string_object_2 = obj2
                .downcast_ref::<StringObject>()
                .expect("The object needs to be of type `StringObject`");

            let str_1 = string_object_1.string();
            let str_2 = string_object_2.string();

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

    fn setup_drun_model(
        entry: &gtk::Entry,
        desktop_entries: &Vec<DesktopEntry>,
    ) -> SingleSelection {
        let model = Self::application_list(desktop_entries);

        Self::setup_model_base(entry, model)
    }

    fn setup_run_model(entry: &gtk::Entry) -> impl IsA<SelectionModel> {
        let model = list_executables_from_path();

        Self::setup_model_base(entry, model)
    }

    fn create_configured_application_window(
        application: &gtk::Application,
    ) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);
        window.init_layer_shell();
        window.set_size_request(600, 400);

        window.set_widget_name("launcher");

        window.set_namespace(Some("chameleon-launcher"));
        window.set_exclusive_zone(-1);
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(if true {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::OnDemand
        });

        // window.set_anchor(Edge::Left, true);
        // window.set_anchor(Edge::Right, true);
        // window.set_anchor(Edge::Top, true);
        // window.set_anchor(Edge::Bottom, true);

        let controller = EventControllerKey::new();
        controller.connect_key_pressed(clone!(
            #[strong]
            window,
            move |_, key, _, _| {
                match key {
                    Key::Escape => {
                        window.set_visible(false);
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
        ));
        window.add_controller(controller);

        window.present();
        window.set_visible(false);

        window
    }
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match fs::metadata(path) {
        Ok(metadata) => {
            let mode = metadata.permissions().mode();
            // Проверяем любой из битов исполнения (owner/group/other)
            mode & 0o111 != 0
        }
        Err(e) => {
            log::error!("{e}");
            false
        }
    }
}

fn list_executables_from_path() -> StringList {
    let path_var = var("PATH").unwrap();

    let mut names: BTreeSet<String> = BTreeSet::new();

    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let path = Path::new(dir);
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for maybe_entry in entries {
            let entry = match maybe_entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let file_path = entry.path();

            if !is_executable(&file_path) {
                continue;
            }

            if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                names.insert(name.to_string());
            }
        }
    }

    names.into_iter().collect()
}
