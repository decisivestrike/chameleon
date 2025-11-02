use grapes::{Component, GtkCompatible, gtk};
use serde::Deserialize;

#[derive(Clone, GtkCompatible)]
pub struct Clock {
    #[root]
    label: gtk::Label,
}

impl Component for Clock {
    type Message = ();

    type Props = ClockConfig;

    fn new(props: Self::Props) -> Self {
        todo!()
    }

    fn update(&self, message: Self::Message) {
        todo!()
    }
}

#[derive(Debug, Deserialize)]
pub struct ClockConfig {
    pub format: Option<String>,
}
