pub mod clock;
pub use clock::Clock;

pub mod battery;
pub use battery::Battery;

pub mod workspaces;
pub use workspaces::Workspaces;

pub mod keyboard_layout;
pub use keyboard_layout::KeyboardLayout;

use crate::common::Metadata;
use grapes::Component;
use gtk::glib::clone::Downgrade;
use std::rc::Rc;
use tokio::sync::watch;

/// Panel module factory
pub trait ModuleFactory {
    type Config;

    fn create(
        config: &Self::Config,
        meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>>;
}

use glib::Object;
use gtk::glib::{self, WeakRef};
use gtk::prelude::ButtonExt;

glib::wrapper! {
    pub struct BaseModule(ObjectSubclass<imp::BaseModule>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl BaseModule {
    pub fn new<Label>(state: watch::Receiver<Label>) -> Self
    where
        Label: AsRef<str> + 'static,
    {
        let module: Self = Object::builder().build();
        glib::spawn_future_local(Self::track_updates(
            module.downgrade(),
            state,
        ));

        module
    }

    async fn track_updates<Label>(
        weak_module: WeakRef<BaseModule>,
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
                    Err(e) => log::error!("{e}"),
                }
            }
        }
    }
}

mod imp {
    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::BaseModule)]
    pub struct BaseModule {}

    #[glib::object_subclass]
    impl ObjectSubclass for BaseModule {
        const NAME: &'static str = "MyGtkAppCustomButton";
        type Type = super::BaseModule;
        type ParentType = gtk::Button;
    }

    impl ObjectImpl for BaseModule {}

    impl WidgetImpl for BaseModule {}

    impl ButtonImpl for BaseModule {}
}
