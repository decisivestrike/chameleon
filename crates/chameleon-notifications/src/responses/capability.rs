use serde::Serialize;
use zbus::zvariant::Type;

#[derive(Serialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s", rename_all = "kebab-case")]
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
