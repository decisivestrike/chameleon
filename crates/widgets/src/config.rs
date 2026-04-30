use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Configuration {
    a: String,
}
