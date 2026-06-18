use gtk::glib::Object;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;
use gtk::{ScrolledWindow, ToggleButton, glib};
use std::cell::RefCell;
use std::iter::successors;

mod imp {
    use super::*;
    use gtk::Orientation;
    use gtk::prelude::OrientableExt;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Default)]
    pub struct ProviderSwitcherImp {
        pub index: Cell<usize>,
        pub active_index: Cell<usize>,
        pub active_button: RefCell<gtk::ToggleButton>,
        pub stack: OnceCell<gtk::Stack>,
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
            box_.set_can_focus(false);
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
        let _ = self.imp().stack.set(stack.clone());
    }

    pub fn append_page(&self, name: &str, page: &gtk::ScrolledWindow) {
        let index = self.index();
        self.set_index(index + 1);

        let button = gtk::ToggleButton::with_label(name);
        button.connect_clicked({
            let switcher = self.clone();
            let page = page.clone();

            move |button| {
                switcher.set_active_index(index);
                switcher.set_active_page(&page);
                switcher.set_active_button(button);
            }
        });

        self.append(&button);
        self.stack().add_child(page);

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

    pub fn active_index(&self) -> usize {
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
        successors(self.first_child(), |child| child.next_sibling())
            .nth(index)
            .expect("must exist")
            .downcast()
            .expect("must be a ToggleButton")
    }

    fn set_active_page(&self, page: &ScrolledWindow) {
        self.stack().set_visible_child(page);
    }

    fn page_by_index(&self, index: usize) -> ScrolledWindow {
        let pages = self.stack().pages();

        let stack_page = pages
            .item(index as u32)
            .and_downcast::<gtk::StackPage>()
            .expect("it must be a StackPage");

        stack_page
            .child()
            .downcast::<ScrolledWindow>()
            .expect("it must be a ScrolledWindow")
    }

    fn stack(&self) -> &gtk::Stack {
        self.imp().stack.get().expect("must be initialized")
    }
}

impl Default for ProviderSwitcher {
    fn default() -> Self {
        Self::new()
    }
}
