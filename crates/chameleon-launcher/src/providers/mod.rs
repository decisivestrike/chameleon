pub mod applications;
pub mod base;
pub mod factory;
pub mod utils;
pub mod view;
pub mod wallpapers;

pub use wallpapers::WallpapersProvider;

use gtk::glib::object::IsA;
use gtk::glib::{self};

pub trait ItemData: IsA<glib::Object> {
    fn id(&self) -> String;

    fn fuzzy_score(&self) -> i32;

    fn set_fuzzy_score(&self, score: i32);
}

pub trait ItemView: IsA<gtk::Widget> {
    type Data: ItemData;

    fn bind(&self, data: &Self::Data);
}

// -----

pub trait Provider {
    fn name(&self) -> &'static str;

    fn update_model(&self, query: &str);

    fn select_below(&self);

    fn len(&self) -> usize;

    fn view(&self) -> gtk::ListBase;

    fn invoke_action(&self) -> bool;

    fn reset_selection(&self);
}

// Allows to bind data
pub trait Bindable: IsA<gtk::Widget> {
    type Data: IsA<glib::Object>;

    fn bind_data(&self, data: &Self::Data);
}
