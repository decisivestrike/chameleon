use gtk::glib;
use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;
    use gtk::Orientation;
    use gtk::prelude::OrientableExt;
    use gtk::subclass::prelude::*;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct ProviderSwitcherImp {
        pub index: Cell<usize>,
        pub active_index: Cell<usize>,

        pub active_page: RefCell<gtk::ScrolledWindow>,
        pub pages: RefCell<Vec<gtk::ScrolledWindow>>,

        pub active_button: RefCell<gtk::Button>,
        pub buttons: RefCell<Vec<gtk::Button>>,

        pub stack: RefCell<Option<gtk::Stack>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProviderSwitcherImp {
        const NAME: &'static str = "ChameleonLauncherProviderSwitcher";
        type Type = super::ProviderSwitcher;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ProviderSwitcherImp {
        fn constructed(&self) {
            self.parent_constructed();

            let box_ = self.obj();
            box_.set_orientation(Orientation::Horizontal);
            box_.set_spacing(0);
            box_.set_widget_name("switcher");
        }
    }

    impl WidgetImpl for ProviderSwitcherImp {}

    impl BoxImpl for ProviderSwitcherImp {}
}

glib::wrapper! {
    pub struct ProviderSwitcher(ObjectSubclass<imp::ProviderSwitcherImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ProviderSwitcher {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_stack(&self, stack: &gtk::Stack) {
        *self.imp().stack.borrow_mut() = Some(stack.clone());
    }

    pub fn append_page(&self, name: &str, page: &gtk::ScrolledWindow) {
        let index = self.imp().index.get();
        self.imp().index.set(index + 1);

        let button = gtk::Button::with_label(name);

        button.connect_clicked({
            let switcher = self.clone();
            let page = page.clone();

            move |button| {
                let active_button = switcher.imp().active_button.clone();
                active_button.borrow().remove_css_class("active");
                button.add_css_class("active");
                *active_button.borrow_mut() = button.clone();

                switcher.stack().map(|s| s.set_visible_child(&page));
                switcher.imp().active_index.set(index);
            }
        });

        self.append(&button);
        self.imp().buttons.borrow_mut().push(button.clone());
        self.imp().pages.borrow_mut().push(page.clone());
        self.add_to_stack(page, name);
    }

    pub fn next_provider(&self) {
        let imp = self.imp();

        let active_index = imp.active_index.get();
        let next_index = (active_index + 1) % imp.index.get();

        imp.active_index.set(next_index);
        self.stack()
            .map(|s| s.set_visible_child(&imp.pages.borrow()[next_index]));

        let active_button = self.imp().active_button.clone();
        active_button.borrow().remove_css_class("active");

        let button = &self.imp().buttons.borrow()[next_index];
        button.add_css_class("active");
        *active_button.borrow_mut() = button.clone();
    }

    fn stack(&self) -> Option<gtk::Stack> {
        self.imp().stack.borrow().clone()
    }

    fn add_to_stack(&self, child: &impl IsA<gtk::Widget>, name: &str) {
        if let Some(stack) = self.imp().stack.borrow_mut().as_mut() {
            stack.add_named(child, Some(name));
        }
    }
}

impl Default for ProviderSwitcher {
    fn default() -> Self {
        Self::new()
    }
}
