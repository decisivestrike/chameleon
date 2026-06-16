mod row;
pub use row::ApplicationRow;

mod entry;

use crate::config::ApplicationProviderConfig;
use crate::providers::Provider;
use crate::providers::applications::entry::ApplicationEntry;
use crate::providers::factory::Factory;
use crate::providers::utils::{filter, sorter};
use freedesktop_desktop_entry::desktop_entries;
use gtk::glib::object::Cast;
use gtk::prelude::*;
use gtk::{
    CustomFilter, CustomSorter, FilterChange, FilterListModel, ListScrollFlags,
    ListView, SingleSelection, SortListModel, SorterChange, gio,
};
use nucleo::{Matcher, Utf32Str};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::env::home_dir;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tracing::{error, info};

pub struct ApplicationProvider {
    pub store: gio::ListStore,
    pub filter: CustomFilter,
    pub sorter: CustomSorter,
    pub selection_model: SingleSelection,
    pub view: ListView,

    pub last_query_len: Cell<usize>,

    pub config: ApplicationProviderConfig,
    pub matcher: RefCell<Matcher>,
}

impl Provider for ApplicationProvider {
    #[inline]
    fn name(&self) -> &'static str {
        "applications"
    }

    fn update_model(&self, query: &str) {
        let mut matcher = self.matcher.borrow_mut();

        for entry in self.store.iter::<ApplicationEntry>().map(|e| e.unwrap()) {
            let score = matcher
                .fuzzy_match(
                    Utf32Str::Ascii(entry.name().as_bytes()),
                    Utf32Str::Ascii(query.as_bytes()),
                )
                .map(|score| score as i32)
                .unwrap_or(-1);

            entry.set_fuzzy_score(score);
        }

        let query_len = query.len();

        if query_len < self.last_query_len.get() {
            self.filter.changed(FilterChange::LessStrict);
            self.sorter.changed(SorterChange::LessStrict);
        } else {
            self.filter.changed(FilterChange::MoreStrict);
            self.sorter.changed(SorterChange::MoreStrict);
        }

        self.last_query_len.set(query_len);
        self.reset_selection()
    }

    fn view(&self) -> gtk::ListBase {
        self.view.clone().upcast()
    }

    fn invoke_action(&self) -> bool {
        let maybe_entry = self
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
        if self.len() > 0 {
            self.selection_model.set_selected(0);
            self.view.scroll_to(0, ListScrollFlags::SELECT, None);
        }
    }

    fn len(&self) -> usize {
        self.selection_model.n_items() as usize
    }

    fn select_below(&self) {
        let i = self.selection_model.selected() + 1;

        if (i as usize) < self.len() {
            self.selection_model.set_selected(i);
            self.view.scroll_to(i, ListScrollFlags::SELECT, None);
        }
    }
}

impl ApplicationProvider {
    pub fn new(config: ApplicationProviderConfig) -> Self {
        let store: gio::ListStore =
            Self::find_apps(&["en".to_string()]).into_iter().collect();

        // filter
        let filter =
            filter::<ApplicationEntry>(|entry| entry.fuzzy_score() != -1);

        let filter_model = FilterListModel::builder()
            .model(&store)
            .filter(&filter)
            .build();

        // sort
        let sorter = sorter::<ApplicationEntry>(move |first, second| {
            let first_score = first.fuzzy_score();
            let second_score = second.fuzzy_score();

            match second_score.cmp(&first_score) {
                Ordering::Equal => {
                    let first_name = first.name();
                    let second_name = second.name();

                    first_name.cmp(&second_name)
                }
                score => score,
            }
        });
        let sort_model =
            SortListModel::new(Some(filter_model), Some(sorter.clone()));

        let selection_model = SingleSelection::new(Some(sort_model.clone()));

        let view = ListView::builder()
            .model(&selection_model)
            .factory(&Factory::new::<ApplicationEntry, ApplicationRow>())
            .build();

        Self {
            selection_model,
            view,
            store,
            filter,
            sorter,
            config,
            matcher: Default::default(),
            last_query_len: Default::default(),
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
