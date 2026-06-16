use gtk::glib::object::{Cast, ObjectType};
use gtk::{GridView, ListScrollFlags, ListView, ScrollInfo};

#[derive(Clone)]
pub enum View {
    List(ListView),
    Grid(GridView),
}

impl View {
    pub fn upcast<T>(&self) -> T
    where
        T: ObjectType,
    {
        match self.clone() {
            View::List(list_view) => unsafe { list_view.unsafe_cast() },
            View::Grid(grid_view) => unsafe { grid_view.unsafe_cast() },
        }
    }

    pub fn scroll_to(
        &self,
        pos: u32,
        flags: ListScrollFlags,
        scroll: Option<ScrollInfo>,
    ) {
        match self {
            View::List(list_view) => list_view.scroll_to(pos, flags, scroll),
            View::Grid(grid_view) => grid_view.scroll_to(pos, flags, scroll),
        };
    }
}
