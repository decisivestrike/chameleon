use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Keyboard {
    pub name: String,
    pub main: bool,
    pub active_keymap: String,
}

#[derive(Debug, Deserialize)]
pub struct Devices {
    pub keyboards: Vec<Keyboard>,
}
