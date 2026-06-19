pub mod applications;
pub mod base;
pub mod factory;
pub mod utils;
pub mod view;
pub mod wallpapers;

pub use wallpapers::WallpapersProvider;

use gtk::gdk::Key;
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

pub trait Provider {
    fn name(&self) -> &'static str;

    fn update_model(&self, query: &str, provider_changed: bool);

    fn move_selection(&self, direction: Direction);

    fn len(&self) -> usize;

    fn view(&self) -> gtk::ListBase;

    fn invoke_action(&self) -> bool;

    fn reset_selection(&self);
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TryFrom<Key> for Direction {
    type Error = ();

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        if !matches!(key, Key::Up | Key::Right | Key::Down | Key::Left) {
            return Err(());
        }

        let direction = match key {
            Key::Up => Direction::Up,
            Key::Right => Direction::Right,
            Key::Down => Direction::Down,
            Key::Left => Direction::Left,
            _ => unreachable!("already checked"),
        };

        Ok(direction)
    }
}
