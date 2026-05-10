use gtk::Orientation;
use gtke::Component;

use crate::config::{Position, config};

#[derive(Component)]
pub struct Separator {
    #[root]
    inner: gtk::Separator,
}

impl Separator {
    pub fn new() -> Self {
        let orientation = match config().position {
            Position::Top | Position::Bottom => Orientation::Vertical,
            Position::Right | Position::Left => Orientation::Horizontal,
        };

        Self {
            inner: gtk::Separator::new(orientation),
        }
    }
}
