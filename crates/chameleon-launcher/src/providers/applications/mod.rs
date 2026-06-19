mod entry;
mod row;

pub use row::ApplicationRow;

use crate::config::ApplicationProviderConfig;
use crate::providers::applications::entry::ApplicationEntry;
use crate::providers::base::ProviderBase;
use crate::providers::factory::Factory;
use crate::providers::view::View;
use crate::providers::{Direction, Provider};
use freedesktop_desktop_entry::desktop_entries;
use gtk::glib::object::CastNone;
use gtk::{ListView, gio};
use std::env::home_dir;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tracing::{error, info};

pub struct ApplicationProvider {
    pub base: ProviderBase<ApplicationEntry>,
    pub config: ApplicationProviderConfig,
}

impl Provider for ApplicationProvider {
    fn name(&self) -> &'static str {
        "Applications"
    }

    fn update_model(&self, query: &str, provider_changed: bool) {
        self.base.update_model(query, provider_changed);
    }

    fn move_selection(&self, direction: Direction) {
        let model = &self.base.selection_model;
        let i = model.selected();
        let len = self.len() as u32;

        let new_i = match direction {
            Direction::Up if i > 0 => i - 1,
            Direction::Down if i + 1 < len => i + 1,
            _ => i,
        };

        self.base.select(new_i);
    }

    fn len(&self) -> usize {
        self.base.len()
    }

    fn view(&self) -> gtk::ListBase {
        self.base.view()
    }

    fn invoke_action(&self) -> bool {
        let maybe_entry = self
            .base
            .selection_model
            .selected_item()
            .and_downcast::<ApplicationEntry>();

        info!("Entry: {:?}", maybe_entry.as_ref().map(|e| e.name()));

        match maybe_entry {
            Some(entry) => {
                self.open_app(&entry.exec(), entry.terminal());
                true
            }
            None => false,
        }
    }

    fn reset_selection(&self) {
        self.base.reset_selection();
    }
}

impl ApplicationProvider {
    pub fn new(config: ApplicationProviderConfig) -> Self {
        let store: gio::ListStore =
            Self::find_apps(&["en".to_string()]).into_iter().collect();

        let base = ProviderBase::new(store, |m| {
            let list_view = ListView::builder()
                .model(m)
                .factory(&Factory::new::<ApplicationEntry, ApplicationRow>())
                .build();

            View::List(list_view)
        });

        Self { base, config }
    }

    fn find_apps(locales: &[String]) -> Vec<ApplicationEntry> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry).ok()
            })
            .collect()
    }

    fn open_app(&self, name: &String, is_terminal: bool) {
        let mut command =
            if is_terminal && let Some(cmd) = &self.config.terminal_cmd {
                let mut command = Command::new(&cmd);
                command.arg(&name);

                command
            } else {
                let mut command = Command::new("sh");
                command.arg("-c").arg(&name);

                command
            };

        let command = command
            .current_dir(home_dir().expect("can get $HOME"))
            .env_remove("RUST_LOG")
            .env_remove("RUST_BACKTRACE")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if self.config.detach {
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        error!("setsid")
                    }

                    Ok(())
                });
            }
        }

        match command.spawn() {
            Ok(_) => info!("App '{name}' spawned"),
            Err(e) => error!("Can't spawn '{name}'. Error: {e}"),
        }
    }
}
