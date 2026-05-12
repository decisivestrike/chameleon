use serde::Serialize;
use zbus::zvariant::{OwnedValue, Type};

#[derive(Serialize, PartialEq, Eq, Type, OwnedValue)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum Capability {
    ActionIcons,
    Actions,
    Body,
    BodyHyperlinks,
    BodyImages,
    BodyMarkup,
    IconMulti,
    IconStatic,
    Persistence,
    Sound,
}
