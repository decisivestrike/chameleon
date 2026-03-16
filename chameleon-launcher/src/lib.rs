use freedesktop_desktop_entry::desktop_entries;
use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    self, EventControllerKey, FilterChange, FilterListModel, Label, ListItem,
    ListScrollFlags, ListView, PolicyType, ScrolledWindow,
    SignalListItemFactory, SingleSelection, SortListModel, SorterChange,
    StringList, Widget,
};
use grapes::layer_shell::{KeyboardMode, Layer, LayerShell};
use grapes::prelude::{
    BoxExt, Cast, CastNone, EditableExt, EntryExt, FilterExt,
    GObjectPropertyExpressionExt, GtkWindowExt, ListItemExt, SorterExt,
    WidgetExt,
};
use grapes::{WindowComponent, gtk::ApplicationWindow};
use gtk::StringObject;
use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

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
    pub fn new(application: &gtk::Application) -> Rc<Self> {
        // get locale
        let locales = &["ru".to_string()];
        let entries = Self::find_desktop_entries(locales);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let entry = gtk::Entry::builder()
            .placeholder_text("Explore...")
            .hexpand(true)
            .build();

        let (selection_model, list_view) =
            Self::setup_list_view(&entry, &entries);

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .can_focus(false)
            .can_target(false)
            .min_content_width(360)
            .min_content_height(400)
            .overlay_scrolling(true)
            .child(&list_view)
            .build();

        let window = Self::create_application_window(application);

        container.append(&entry);
        container.append(&scrolled_window);

        window.set_child(Some(&container));

        let launcher = Rc::new(Self {
            window: window.clone(),
            entry: entry.clone(),
            selection_model,
            list_view,
            visibility: Cell::new(false),
        });

        entry.connect_activate(clone!(
            #[weak]
            launcher,
            move |_| {
                let selected_item: StringObject = launcher
                    .selection_model
                    .selected_item()
                    .and_downcast()
                    .expect("cant cast");

                let name = selected_item.string().to_string();

                if let Some(exec) = entries.get(&name) {
                    Self::start_app(&exec).expect("cant run");
                }

                launcher.toggle_visibility();
            }
        ));

        let controller = EventControllerKey::new();
        controller.connect_key_pressed(clone!(
            #[strong]
            launcher,
            move |_, key, _, _| {
                match key {
                    Key::Escape => {
                        launcher.toggle_visibility();
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            }
        ));
        window.add_controller(controller);

        launcher
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

    fn find_desktop_entries(locales: &[String]) -> HashMap<String, String> {
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
                let exec = group.entry("Exec")?.to_string();

                Some((name, exec))
            })
            .collect()
    }

    fn setup_list_view(
        entry: &gtk::Entry,
        entries: &HashMap<String, String>,
    ) -> (SingleSelection, ListView) {
        let names = entries.keys().map(|k| k.clone()).collect();

        let selection_model = Self::setup_selection_model(entry, names);
        let item_factory = Self::create_factory();

        let list_view = ListView::builder()
            .model(&selection_model)
            .factory(&item_factory)
            .build();

        (selection_model, list_view)
    }

    fn create_factory() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let label = Label::new(None);

            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            list_item
                .property_expression("item")
                .chain_property::<StringObject>("string")
                .bind(&label, "label", Widget::NONE);

            list_item.set_child(Some(&label));
        });

        factory
    }

    fn setup_selection_model(
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

    fn create_application_window(
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

        window
    }

    fn start_app(name: impl AsRef<OsStr>) -> io::Result<Child> {
        Command::new("sh")
            .arg("-c")
            .arg(name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }
}
