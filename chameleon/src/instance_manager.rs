use chameleon_config::Config;
use chameleon_config::Panel as PanelConfig;
use chameleon_config::Widgets as WidgetsConfig;
use chameleon_panel::Panel;
use chameleon_widgets::WidgetsLayer;
use grapes::tokio::sync::Mutex;
use grapes::{
    WindowComponent,
    gtk::{self, gdk::Monitor},
    prelude::{MonitorExt, monitor::GrapesMonitorExt},
};
use log::info;
use std::rc::Rc;
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

pub static INSTANCE_MANAGER: LazyLock<Mutex<InstanceManager>> =
    LazyLock::new(|| Mutex::new(InstanceManager::new()));

/// String here is a monitor connector name
#[derive(Default)]
pub struct InstanceManager {
    widgets_layers: HashMap<String, WidgetsLayer>,
    panels: HashMap<String, Panel>,
    monitor_connectors: HashSet<String>,
}

impl InstanceManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn configure_modules(
        &mut self,
        application: &gtk::Application,
        config: &Rc<Config>,
    ) {
        self.configure_panels(application, &config.panel);
        self.configure_widgets_layers(application, &config.widgets);

        info!("Modules configured!");
    }

    fn configure_panels(
        &mut self,
        application: &gtk::Application,
        panel_config: &Rc<PanelConfig>,
    ) {
        match panel_config.enabled {
            true if !self.panels.is_empty() => self
                .panels
                .values_mut()
                .for_each(|panel| panel.configure(panel_config.clone())),
            true => {
                log::info!("Setup bar...");

                for monitor in Monitor::all().iter() {
                    let panel =
                        Panel::new(application, monitor, panel_config.clone());

                    panel.present();

                    let connector_name =
                        monitor.connector().unwrap().to_string();

                    self.panels.insert(connector_name, panel);
                }
            }
            false => {
                if !self.panels.is_empty() {
                    self.panels.retain(|_, panel| {
                        panel.destroy();
                        false
                    })
                }
            }
        }
    }

    fn configure_widgets_layers(
        &self,
        _application: &gtk::Application,
        _widgets_config: &Rc<WidgetsConfig>,
    ) {
        log::info!("Widgets layer temporary unavailable")
    }
}

unsafe impl Sync for InstanceManager {}
unsafe impl Send for InstanceManager {}
