use grapes::glib::{self, clone};
use grapes::gtk::gdk::Key;
use grapes::gtk::{
    EventControllerKey, FilterListModel, GridView, Label, ListItem,
    ListItemFactory, ListView, PolicyType, ScrolledWindow, SelectionModel,
    SignalListItemFactory, SingleSelection, SortListModel, StringList, Widget,
};
use grapes::layer_shell::{KeyboardMode, Layer, LayerShell};
use grapes::prelude::{
    BoxExt, Cast, CastNone, FilterExt, GObjectPropertyExpressionExt,
    GtkWindowExt, IsA, ListItemExt, ListModelExt, WidgetExt,
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

        let model = Self::create_model();
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

    fn create_model() -> impl IsA<SelectionModel> {
        // test data
        let model: StringList =
            (0..=100_000).map(|number| number.to_string()).collect();

        let filter = gtk::CustomFilter::new(move |obj| {
            let string_object = obj
                .downcast_ref::<StringObject>()
                .expect("The object needs to be of type `IntegerObject`.");

            string_object.string().contains('0')
        });
        let filter_model = FilterListModel::new(Some(model), Some(filter));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let string_object_1 = obj1
                .downcast_ref::<StringObject>()
                .expect("The object needs to be of type `StringObject`.");
            let string_object_2 = obj2
                .downcast_ref::<StringObject>()
                .expect("The object needs to be of type `StringObject`.");

            let str_1 = string_object_1.string();
            let str_2 = string_object_2.string();

            str_2.len().cmp(&str_1.len()).into()
        });
        let filter_and_sort_model =
            SortListModel::new(Some(filter_model), Some(sorter.clone()));

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
