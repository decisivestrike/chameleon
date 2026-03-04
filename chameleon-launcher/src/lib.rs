use std::collections::BTreeSet;
use std::env::var;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    EventControllerKey, FilterChange, FilterListModel, GridView, Label,
    ListItem, ListItemFactory, ListView, PolicyType, ScrolledWindow,
    SelectionModel, SignalListItemFactory, SingleSelection, SortListModel,
    SorterChange, StringList, Widget,
};
use grapes::layer_shell::{KeyboardMode, Layer, LayerShell};
use grapes::prelude::{
    BoxExt, Cast, EditableExt, FilterExt, GObjectPropertyExpressionExt,
    GtkWindowExt, IsA, ListItemExt, SorterExt, WidgetExt,
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

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,

    container: gtk::Box,
    entry: gtk::Entry,
    list_view: ListView,
}

impl Launcher {
    pub fn new(application: &gtk::Application) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let entry = gtk::Entry::new();
        container.append(&entry);

        let model = Self::setup_model(&entry);
        let factory = Self::create_factory();

        let list_view = ListView::new(Some(model), Some(factory));

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .min_content_width(360)
            .min_content_height(400)
            .child(&list_view)
            .build();

        container.append(&scrolled_window);

        let window = Self::create_configured_application_window(application);
        window.set_child(Some(&container));

        Self {
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

    fn create_factory() -> impl IsA<ListItemFactory> {
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

    fn setup_model(entry: &gtk::Entry) -> impl IsA<SelectionModel> {
        let model = list_executables_from_path();

        let filter = gtk::CustomFilter::new(clone!(
            #[strong]
            entry,
            move |obj| {
                let string_object = obj
                    .downcast_ref::<StringObject>()
                    .expect("The object needs to be of type `StringObject`");

                string_object.string().contains(&entry.text().to_string())
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
                if key == Key::Escape {
                    window.set_visible(false);
                }

                glib::Propagation::Stop
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
    if let Ok(metadata) = fs::metadata(path) {
        let mode = metadata.permissions().mode();
        // Проверяем любой из битов исполнения (owner/group/other)
        mode & 0o111 != 0
    } else {
        false
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

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let file_path = entry.path();

            if !file_path.is_file() {
                continue;
            }
            if let Ok(metadata) = fs::metadata(&file_path) {
                let mode = metadata.permissions().mode();
                if mode & 0o111 == 0 {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                names.insert(name.to_string());
            }
        }
    }

    names.into_iter().collect()
}
