mod row;
pub use row::ApplicationRow;

mod factory;
pub use factory::Factory;

use crate::config::ApplicationProviderConfig;
use crate::entry_object::ApplicationEntry;
use crate::providers::Provider;
use freedesktop_desktop_entry::desktop_entries;
use gtk::glib::object::Cast;
use gtk::prelude::*;
use gtk::{ListScrollFlags, ListView, SingleSelection, gio};
use nucleo::{Matcher, Utf32Str};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::env::home_dir;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tracing::{error, info};

pub struct ApplicationProvider {
    pub apps: Vec<ApplicationEntry>,
    pub selection_model: SingleSelection,
    pub view: ListView,
    pub store: gio::ListStore,
    pub matcher: RefCell<Matcher>,

    pub config: ApplicationProviderConfig,
}

impl Provider for ApplicationProvider {
    #[inline]
    fn name(&self) -> &'static str {
        "applications"
    }

    fn update_model(&self, query: &str) {
        let mut updated_store: Vec<_> = self
            .apps
            .iter()
            .filter_map(|app| {
                let mut matcher = self.matcher.borrow_mut();

                let score = matcher.fuzzy_match(
                    Utf32Str::Ascii(app.name().as_bytes()),
                    Utf32Str::Ascii(query.as_bytes()),
                )?;

                Some((app.clone(), score))
            })
            .collect();

        updated_store.sort_by(|first, second| {
            let first_score = first.1;
            let second_score = second.1;

            let score_cmp = second_score.cmp(&first_score);

            if let Ordering::Equal = score_cmp {
                let first_name = first.0.name();
                let second_name = second.0.name();

                first_name.cmp(&second_name)
            } else {
                score_cmp
            }
        });

        let updated_store: Vec<_> =
            updated_store.into_iter().map(|e| e.0).collect();

        let len = self.store.n_items();
        self.store.splice(0, len, &updated_store);

        if updated_store.len() > 0 {
            self.view.scroll_to(0, ListScrollFlags::SELECT, None);
        }
    }

    fn view(&self) -> gtk::ListBase {
        self.view.clone().upcast()
    }

    fn invoke_action(&self) -> bool {
        let maybe_entry = self
            .selection_model
            .selected_item()
            .and_downcast::<ApplicationEntry>();

        match maybe_entry {
            Some(entry) => {
                self.open_app(&entry.exec(), entry.terminal());
                true
            }
            None => false,
        }
    }

    fn reset(&self) {
        self.selection_model.set_selected(0);
        self.view.scroll_to(0, ListScrollFlags::FOCUS, None);
    }
}

impl ApplicationProvider {
    pub fn new(config: ApplicationProviderConfig) -> Self {
        let store = gio::ListStore::new::<ApplicationEntry>();
        let selection_model = SingleSelection::new(Some(store.clone()));

        let view = ListView::builder()
            .model(&selection_model)
            .factory(&Factory::new())
            .build();

        Self {
            apps: Self::find_apps(&["en".to_string()]),
            selection_model,
            view,
            store,
            config,
            matcher: Default::default(),
        }
    }

    fn find_apps(locales: &[String]) -> Vec<ApplicationEntry> {
        desktop_entries(locales)
            .into_iter()
            .filter_map(|desktop_entry| {
                ApplicationEntry::try_from(desktop_entry).ok()
            })
            .collect()
    }

    fn open_app(&self, name: &str, is_terminal: bool) {
        let name = name.to_string();

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
