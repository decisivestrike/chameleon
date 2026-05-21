pub mod shared;

use chameleon_hyprland::Hyprland;
use std::env::var;
use std::sync::LazyLock;

pub static COMPOSITOR: LazyLock<Compositor> = LazyLock::new(Compositor::define);

pub enum Compositor {
    Hyprland(Hyprland),
    Unsupported,
}

impl Compositor {
    /// Tries to determine which compositor you are using
    fn define() -> Self {
        if let Ok(his) = var("HYPRLAND_INSTANCE_SIGNATURE") {
            let hyprland = Hyprland::new(his);
            Compositor::Hyprland(hyprland)
        } else {
            Compositor::Unsupported
        }
    }
}
