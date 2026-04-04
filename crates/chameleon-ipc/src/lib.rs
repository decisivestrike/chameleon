pub mod compositor;
pub mod hyprland;

use crate::hyprland::Hyprland;
use std::{env::var, sync::LazyLock};

pub static COMPOSITOR: LazyLock<CompositorVariant> =
    LazyLock::new(CompositorVariant::define);

#[derive(Debug)]
pub enum CompositorVariant {
    Hyprland(Hyprland),
    Niri(()),
    Unknown,
}

impl CompositorVariant {
    /// Tries to determine which compositor you are using
    fn define() -> Self {
        if let Ok(his) = var("HYPRLAND_INSTANCE_SIGNATURE") {
            let hyprland = Hyprland::init(his);

            CompositorVariant::Hyprland(hyprland)
        } else if false {
            CompositorVariant::Niri(())
        } else {
            CompositorVariant::Unknown
        }
    }
}
