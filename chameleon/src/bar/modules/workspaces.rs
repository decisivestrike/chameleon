use chameleon_config::bar::Workspaces as WorkspacesConfig;
use grapes::{
    Component, GtkCompatible,
    gtk::{self, Orientation},
};

#[derive(Clone, Debug, GtkCompatible)]
pub struct Workspaces {
    #[root]
    container: gtk::Box,
}

impl Component for Workspaces {
    const NAME: &str = "workspaces";

    type Props = &'static WorkspacesConfig;

    fn new(config: &WorkspacesConfig) -> Self {
        let container = gtk::Box::new(Orientation::Horizontal, 0);

        Self { container }
    }
}
