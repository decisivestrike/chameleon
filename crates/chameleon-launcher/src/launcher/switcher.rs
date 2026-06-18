use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;
use gtk::{ScrolledWindow, ToggleButton, glib};
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

        pub active_button: RefCell<gtk::ToggleButton>,
        pub buttons: RefCell<Vec<gtk::ToggleButton>>,

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
        let switcher: Self = Object::builder().build();

        switcher
    }

    pub fn set_stack(&self, stack: &gtk::Stack) {
        *self.imp().stack.borrow_mut() = Some(stack.clone());
    }

    pub fn append_page(&self, name: &str, page: &gtk::ScrolledWindow) {
        let index = self.index();
        self.set_index(index + 1);

        let button = gtk::ToggleButton::with_label(name);
        button.connect_clicked({
            let switcher = self.clone();
            let page = page.clone();

            move |button| {
                switcher.set_active_page(&page);
                switcher.set_active_button(button);
                switcher.set_active_index(index);
            }
        });

        self.append(&button);
        self.imp().buttons.borrow_mut().push(button.clone());

        self.add_to_stack(page);
        self.imp().pages.borrow_mut().push(page.clone());

        if index == 0 {
            self.set_active_button(&self.button_by_index(0));
        }
    }

    pub fn next_provider(&self) {
        let active_index = self.active_index();
        let next_index = (active_index + 1) % self.index();

        self.set_active_index(next_index);
        self.set_active_page(&self.page_by_index(next_index));
        self.set_active_button(&self.button_by_index(next_index));
    }

    fn index(&self) -> usize {
        self.imp().index.get()
    }

    fn set_index(&self, value: usize) {
        self.imp().index.set(value);
    }

    fn active_index(&self) -> usize {
        self.imp().active_index.get()
    }

    fn set_active_index(&self, index: usize) {
        self.imp().active_index.set(index);
    }

    fn active_button(&self) -> gtk::ToggleButton {
        self.imp().active_button.borrow().clone()
    }

    fn set_active_button(&self, button: &ToggleButton) {
        let active_button = self.active_button();
        active_button.set_active(false);
        button.set_active(true);
        *self.imp().active_button.borrow_mut() = button.clone();
    }

    fn button_by_index(&self, index: usize) -> ToggleButton {
        self.imp().buttons.borrow()[index].clone()
    }

    fn set_active_page(&self, page: &ScrolledWindow) {
        self.stack().map(|s| s.set_visible_child(page));
    }

    fn page_by_index(&self, index: usize) -> ScrolledWindow {
        self.imp().pages.borrow()[index].clone()
    }

    fn stack(&self) -> Option<gtk::Stack> {
        self.imp().stack.borrow().clone()
    }

    fn add_to_stack(&self, child: &impl IsA<gtk::Widget>) {
        if let Some(stack) = self.imp().stack.borrow_mut().as_mut() {
            stack.add_child(child);
        }
    }
}

impl Default for ProviderSwitcher {
    fn default() -> Self {
        Self::new()
    }
}
