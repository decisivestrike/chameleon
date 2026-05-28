mod image_cell;
mod wallpaper;

use crate::providers::factory::Factory;
use crate::providers::utils::{filter, sorter};
use crate::providers::wallpapers::image_cell::ImageCell;
use crate::providers::wallpapers::wallpaper::Wallpaper;
use gtk::gio::ListStore;
use gtk::gio::prelude::{ListModelExt, ListModelExtManual};
use gtk::glib::clone;
use gtk::glib::object::{Cast, CastNone};
use gtk::prelude::{FilterExt, SorterExt};
use gtk::{
    Align, CustomFilter, CustomSorter, FilterChange, FilterListModel, GridView,
    ListScrollFlags, SingleSelection, SortListModel, SorterChange, gio, glib,
};
use nucleo::{Matcher, Utf32Str};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub struct WallpapersProvider {
    pub store: gio::ListStore,
    pub filter: CustomFilter,
    pub sorter: CustomSorter,
    pub selection_model: SingleSelection,
    pub view: GridView,

    pub last_query_len: Cell<usize>,

    pub matcher: RefCell<Matcher>,
}

fn get_image_paths(root: &str) -> Vec<PathBuf> {
    let extensions = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "svg"];

    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| extensions.contains(&s.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

impl WallpapersProvider {
    pub fn new() -> Self {
        let store: gio::ListStore = ListStore::new::<Wallpaper>();

        let paths = get_image_paths("/home/inqlog/Pictures/wallpapers/pixel");

        glib::idle_add_local_once(clone!(
            #[strong]
            store,
            move || {
                for path in paths.into_iter() {
                    glib::idle_add_local_once(clone!(
                        #[strong]
                        store,
                        move || {
                            let wallpaper = Wallpaper::new(path);
                            store.append(&wallpaper);
                        }
                    ));
                }
            }
        ));

        // filter
        let filter = filter::<Wallpaper>(|entry| entry.fuzzy_score() != -1);

        let filter_model = FilterListModel::builder()
            .model(&store)
            .filter(&filter)
            .build();

        // sort
        let sorter = sorter::<Wallpaper>(move |first, second| {
            let first_score = first.fuzzy_score();
            let second_score = second.fuzzy_score();

            match second_score.cmp(&first_score) {
                Ordering::Equal => {
                    let first_name = first.picture_name();
                    let second_name = second.picture_name();

                    first_name.cmp(&second_name)
                }
                score => score,
            }
        });
        let sort_model =
            SortListModel::new(Some(filter_model), Some(sorter.clone()));

        let selection_model = SingleSelection::new(Some(sort_model.clone()));
        let view = GridView::builder()
            .min_columns(5)
            .max_columns(5)
            .halign(Align::Fill)
            .valign(Align::Fill)
            .model(&selection_model)
            .factory(&Factory::new::<Wallpaper, ImageCell>())
            .build();

        Self {
            selection_model,
            view,
            store,
            filter,
            sorter,
            matcher: Default::default(),
            last_query_len: Default::default(),
        }
    }
}

impl super::Provider for WallpapersProvider {
    fn name(&self) -> &'static str {
        "wallpapers"
    }

    fn update_model(&self, query: &str) {
        let mut matcher = self.matcher.borrow_mut();

        for wallpaper in self.store.iter::<Wallpaper>().map(|e| e.unwrap()) {
            let score = matcher
                .fuzzy_match(
                    Utf32Str::Ascii(wallpaper.picture_name().as_bytes()),
                    Utf32Str::Ascii(query.as_bytes()),
                )
                .map(|score| score as i32)
                .unwrap_or(-1);

            wallpaper.set_fuzzy_score(score);
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
        self.reset()
    }

    fn view(&self) -> gtk::ListBase {
        self.view.clone().upcast()
    }

    fn invoke_action(&self) -> bool {
        let maybe_path = self
            .selection_model
            .selected_item()
            .and_downcast::<Wallpaper>();

        match maybe_path {
            Some(path) => {
                Command::new("awww")
                    .arg("img")
                    .args(["--transition-fps", "120"])
                    .args(["--transition-type", "wipe"])
                    .args(["--transition-angle", "30"])
                    .args(["--transition-duration", "0.8"])
                    .args(["--transition-step", "40"])
                    .args(["--transition-bezier", "0.42,0.0,1.0,1.0"])
                    .arg(format!(
                        "/home/inqlog/Pictures/wallpapers/pixel/{}",
                        path.picture_name()
                    ))
                    .spawn()
                    .expect("awww command failed to start");

                true
            }
            None => false,
        }
    }

    fn reset(&self) {
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
