use grapes::{Component, gtk};

mod event;
mod factory;

#[derive(Component)]
pub struct KeyboardLayout {
    #[root]
    root: gtk::Label,
}
