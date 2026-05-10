pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;
pub use workspaces::Workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

pub mod pulseaudio;
pub use pulseaudio::Pulseaudio;

use crate::common::Metadata;
use gtk::glib::clone::Downgrade;
use gtk::glib::{self, WeakRef};
use gtke::Component;
use std::borrow::Borrow;
use std::rc::Rc;
use tokio::sync::watch;
use tracing::error;

/// Panel module factory
pub trait ModuleFactory {
    type Config;

    fn create(
        config: &Self::Config,
        meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>>;
}

#[derive(Debug, Component)]
pub struct BaseModule {
    #[root]
    label: gtk::Label,
}

impl BaseModule {
    pub fn new<Label>(state: watch::Receiver<Label>) -> Self
    where
        Label: AsRef<str> + 'static,
    {
        let label = gtk::Label::new(None);
        glib::spawn_future_local(Self::track_updates(label.downgrade(), state));

        Self { label }
    }

    async fn track_updates<Label>(
        weak_module: WeakRef<gtk::Label>,
        mut state: watch::Receiver<Label>,
    ) where
        Label: AsRef<str> + 'static,
    {
        loop {
            if let Some(module) = weak_module.upgrade() {
                match state.changed().await {
                    Ok(_) => {
                        let updated_label = state.borrow();
                        module.set_label(updated_label.as_ref());
                    }
                    Err(e) => error!("{e}"),
                }
            } else {
                break;
            }
        }
    }
}

impl Borrow<gtk::Widget> for BaseModule {
    fn borrow(&self) -> &gtk::Widget {
        self.label.borrow()
    }
}
