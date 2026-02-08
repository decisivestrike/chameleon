use chameleon_config::{Config, PanelConfig, WidgetsConfig};
use chameleon_panel::Panel;
use chameleon_widgets::WidgetsLayer;
use dashmap::DashMap;
use grapes::{
    WindowComponent,
    gtk::{self, gdk::Monitor},
    prelude::{MonitorExt, monitor::GrapesMonitorExt},
};
use std::{rc::Rc, sync::LazyLock};

/// Global instance manager
pub static INSTANCE_MANAGER: LazyLock<InstanceManager> =
    LazyLock::new(|| Default::default());

/// String here is a monitor connector name
#[derive(Default)]
pub struct InstanceManager {
    widgets_layers: DashMap<String, WidgetsLayer>,
    panels: DashMap<String, Panel>,
}

impl InstanceManager {
    pub fn monitor_connector_of_panel(&self, panel: &Panel) -> Option<String> {
        self.panels
            .iter()
            .find(|pair| pair.value() == panel)
            .map(|pair| pair.key().clone())
    }

    pub fn configure_modules(
        &self,
        application: &gtk::Application,
        config: &Rc<Config>,
    ) {
        self.configure_panels(application, &config.panel);
        self.configure_widgets_layers(application, &config.widgets);

        log::info!("Modules configured!");
    }

    /// Просто удаляем все панельки. Если они включены, то снова создаем
    fn configure_panels(
        &self,
        application: &gtk::Application,
        panel_config: &Rc<PanelConfig>,
    ) {
        if !self.panels.is_empty() {
            self.panels.retain(|_, panel| {
                panel.destroy();
                false
            })
        }

        if panel_config.enabled {
            log::info!("Setup panels...");

            for monitor in Monitor::all().iter() {
                let panel =
                    Panel::new(application, monitor, panel_config.clone());
                panel.present();

                let connector_name = monitor.connector().unwrap().to_string();
                self.panels.insert(connector_name, panel);
            }
        }
    }

    fn configure_widgets_layers(
        &self,
        application: &gtk::Application,
        widgets_config: &Rc<WidgetsConfig>,
    ) {
        if !self.widgets_layers.is_empty() {
            self.widgets_layers.retain(|_, wl| {
                wl.destroy();
                false
            })
        }

        if widgets_config.enabled {
            log::info!("Setup widgets...");

            for monitor in Monitor::all().iter() {
                let widgets_layer = WidgetsLayer::new(
                    application,
                    monitor,
                    widgets_config.clone(),
                );
                widgets_layer.present();

                let connector_name = monitor.connector().unwrap().to_string();
                self.widgets_layers.insert(connector_name, widgets_layer);
            }
        }
    }
}

unsafe impl Sync for InstanceManager {}
unsafe impl Send for InstanceManager {}
