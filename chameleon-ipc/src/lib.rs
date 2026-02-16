pub mod compositor;
pub mod hyprland;

use crate::{compositor::Compositor, hyprland::Hyprland};
use std::{env::var, sync::LazyLock};

pub static COMPOSITOR: LazyLock<CompositorVariant> =
    LazyLock::new(CompositorVariant::define);

pub enum CompositorVariant {
    Hyprland(Hyprland),
    Niri(()),
    Unknown,
}

impl CompositorVariant {
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

    /// Starts a task that listens to the compositor's events
    pub fn run(&'static self) {
        match self {
            CompositorVariant::Hyprland(hyprland) => hyprland.run(),
            CompositorVariant::Niri(_) => unimplemented!("not yet"),
            CompositorVariant::Unknown => (),
        }
    }
}
