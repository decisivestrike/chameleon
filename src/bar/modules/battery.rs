use grapes::{Component, GtkCompatible, gtk};
use serde::Deserialize;

#[derive(Clone, GtkCompatible)]
pub struct Battery {
    #[root]
    label: gtk::Label,
}

impl Component for Battery {
    const NAME: &str = "battery";

    type Message = u8;
    type Props = &'static BatteryConfig;

    fn new(props: Self::Props) -> Self {
        todo!()
    }

    fn update(&self, message: Self::Message) {
        todo!()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BatteryConfig {
    pub icons: Vec<String>,
    pub name: String,
    pub format: String,
}
