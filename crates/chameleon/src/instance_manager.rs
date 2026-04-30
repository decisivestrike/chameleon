// use chameleon_launcher::Launcher;
// use chameleon_panel::Panel;
// use dashmap::DashMap;
// use grapes::gtk::gdk::Monitor;
// use grapes::gtk::{self};
// use grapes::prelude::MonitorExt;
// use grapes::prelude::monitor::GrapesMonitorExt;
// use grapes::tokio::fs;
// use grapes::tokio::io::{AsyncBufReadExt, BufReader};
// use grapes::tokio::net::UnixListener;
// use grapes::tokio::sync::RwLock;
// use grapes::{RT, WindowComponent, glib};
// use std::path::Path;
// use std::rc::Rc;
// use std::sync::LazyLock;
// use widgets::WidgetsLayer;

// /// Global instance manager
// pub static INSTANCE_MANAGER: LazyLock<InstanceManager> =
//     LazyLock::new(InstanceManager::default);

// const SOCKET_PATH: &str = "/tmp/chameleon-recv.sock";

// /// Handles monitors connection/disconnection
// ///
// /// String here is a monitor connector name
// pub struct InstanceManager {
//     widgets_layers: DashMap<String, WidgetsLayer>,
//     panels: DashMap<String, Panel>,
// }

// impl InstanceManager {
//     pub fn configure_modules(
//         &self,
//         app: &gtk::Application,
//         config: &'static Config,
//     ) {
//         self.configure_panels(app, &config.panel);
//         self.configure_widgets_layers(app, &config.widgets);
//         self.configure_launcher(app, &config.launcher);

//         log::info!("Modules configured!");
//     }

//     pub fn toggle_launcher(&self) {
//         if let Some(launcher) = &*self.launcher.blocking_read() {
//             launcher.toggle_visibility();
//         }
//     }

//     /// Просто удаляем все панельки. Если они включены, то снова создаем
//     fn configure_panels(
//         &self,
//         application: &gtk::Application,
//         panel_config: &PanelConfig,
//     ) {
//         if !self.panels.is_empty() {
//             self.panels.retain(|_, panel| {
//                 panel.destroy();
//                 false
//             })
//         }

//         if panel_config.enabled {
//             log::info!("Setup panels...");

//             for monitor in Monitor::all().iter() {
//                 let panel = Panel::new(application, monitor, panel_config);
//                 panel.present();

//                 let connector_name =
// monitor.connector().unwrap().to_string();                 
// self.panels.insert(connector_name, panel);             }
//         }
//     }

//     fn configure_widgets_layers(
//         &self,
//         application: &gtk::Application,
//         widgets_config: &WidgetsConfig,
//     ) {
//         if !self.widgets_layers.is_empty() {
//             self.widgets_layers.retain(|_, wl| {
//                 wl.destroy();
//                 false
//             })
//         }

//         if widgets_config.enabled {
//             log::info!("Setup widgets...");

//             for monitor in Monitor::all().iter() {
//                 let widgets_layer =
//                     WidgetsLayer::new(application, monitor, widgets_config);
//                 widgets_layer.present();

//                 let connector_name =
// monitor.connector().unwrap().to_string();                 
// self.widgets_layers.insert(connector_name, widgets_layer);             }
//         }
//     }

//     async fn listen_socket() {
//         let path = Path::new(SOCKET_PATH);

//         if path.exists() {
//             fs::remove_file(SOCKET_PATH).await.unwrap();
//         }

//         let listener = UnixListener::bind(SOCKET_PATH).unwrap();
//         log::info!("Listening '{SOCKET_PATH}'");

//         loop {
//             let (stream, _addr) = listener.accept().await.unwrap();
//             let mut lines = BufReader::new(stream).lines();

//             match lines.next_line().await {
//                 Ok(Some(_line)) => {
//                     glib::idle_add(|| {
//                         INSTANCE_MANAGER.toggle_launcher();
//                         glib::ControlFlow::Break
//                     });
//                 }
//                 _ => (),
//             }
//         }
//     }
// }

// impl Default for InstanceManager {
//     fn default() -> Self {
//         let im = Self {
//             widgets_layers: Default::default(),
//             panels: Default::default(),
//             launcher: None.into(),
//         };

//         RT.spawn(Self::listen_socket());

//         im
//     }
// }

// unsafe impl Sync for InstanceManager {}
// unsafe impl Send for InstanceManager {}
