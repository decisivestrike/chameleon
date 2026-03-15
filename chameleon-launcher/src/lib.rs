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
use std::collections::HashMap;
use std::io;
use std::process::{Child, Command, Stdio};

#[derive(WindowComponent)]
pub struct Launcher {
    #[root]
    window: ApplicationWindow,
}

impl Launcher {
    pub fn new(application: &gtk::Application) -> Self {
        // get locale
        let locales = &["ru".to_string()];

        let desktop_entries: HashMap<_, _> = desktop_entries(locales)
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
            .collect();

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let entry = gtk::Entry::new();
        container.append(&entry);

        let selection_model = Self::setup_model(
            &entry,
            desktop_entries.keys().map(|k| k.clone()).collect(),
        );
        let factory = Self::create_factory();
        let list_view =
            ListView::new(Some(selection_model.clone()), Some(factory));

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

        container.append(&scrolled_window);

        let window = Self::create_configured_application_window(application);
        window.set_child(Some(&container));

        entry.connect_activate({
            let selection_model = selection_model.clone();
            let window = window.clone();
            let list_view = list_view.clone();
            move |entry| {
                let selected_item: StringObject = selection_model
                    .selected_item()
                    .and_downcast()
                    .expect("cant cast");

                let name = selected_item.string();

                if let Some(exec) = desktop_entries.get(&name.to_string()) {
                    Self::start_app(&exec).expect("cant run");
                }

                window.set_visible(false);
                entry.set_text("");
                selection_model.set_selected(0);
                list_view.scroll_to(0, ListScrollFlags::FOCUS, None);
            }
        });

        Self { window }
    }

    pub fn toggle_visibility(&self) {
        let current_visibility = self.window.is_visible();
        self.window.set_visible(!current_visibility);

        // if !current_visibility {
        //     self.window.grab_focus();
        // }
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

    fn setup_model(entry: &gtk::Entry, model: StringList) -> SingleSelection {
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

    // Sync
    fn start_app(name: &str) -> io::Result<Child> {
        Command::new("sh")
            .arg("-c")
            .arg(name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }
}
