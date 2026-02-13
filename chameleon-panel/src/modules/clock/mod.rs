mod factory;
pub use factory::ClockFactory;

use chrono::{DateTime, Local};
use grapes::{Component, State, gtk::prelude::WidgetExt};
use grapes_components::StatefullLabel;
use std::rc::Rc;

#[derive(Debug, Component)]
pub struct Clock {
    #[root]
    label: StatefullLabel<String>,
}

impl Clock {
    const NAME: &str = "clock";

    pub fn new(formatted_time: &Rc<State<String>>) -> Self {
        let label = StatefullLabel::new(formatted_time);
        label.as_ref().set_widget_name(Self::NAME);
        label.as_ref().add_css_class("module");

        Self { label }
    }

    fn format(time: DateTime<Local>, format: &str) -> String {
        time.format(format).to_string()
    }
}
