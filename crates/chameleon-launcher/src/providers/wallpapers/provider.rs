use crate::config::WallpapersProviderConfig;
use crate::providers::base::ProviderBase;
use crate::providers::factory::Factory;
use crate::providers::view::View;
use crate::providers::wallpapers::cache::cache_filename;
use crate::providers::wallpapers::image_cell::ImageCell;
use crate::providers::wallpapers::wallpaper::Wallpaper;
use crate::providers::{Direction, Provider};
use gtk::gio::ListStore;
use gtk::glib::clone;
use gtk::glib::object::CastNone;
use gtk::{Align, GridView, gio, glib};
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct WallpapersProvider {
    pub(super) base: ProviderBase<Wallpaper>,
    pub wallpapers_path: String,
    pub change_wallpapers_cmd: String,
    pub matugen_enabled: bool,
    pub matugen_flags: Vec<String>,
}

impl WallpapersProvider {
    pub fn new(config: WallpapersProviderConfig) -> Self {
        let store: gio::ListStore = ListStore::new::<Wallpaper>();

        let wallpapers_path =
            config.path.as_path().to_string_lossy().to_string();
        let paths = Self::get_image_paths(&wallpapers_path);

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

        let base = ProviderBase::new(store, |m| {
            let grid = GridView::builder()
                .min_columns(5)
                .max_columns(5)
                .halign(Align::Fill)
                .valign(Align::Fill)
                .model(m)
                .factory(&Factory::new::<Wallpaper, ImageCell>())
                .build();

            View::Grid(grid)
        });

        Self {
            base,
            wallpapers_path,
            change_wallpapers_cmd: config.change_command,
            matugen_enabled: config.matugen,
            matugen_flags: config.matugen_flags,
        }
    }

    pub fn call_change_wallpaper_cmd(&self, full_wp_path: &str) {
        let (command, args) =
            self.change_wallpapers_cmd.split_once(" ").unwrap();

        let args: Vec<&str> = args
            .split_whitespace()
            .map(|s| if s == "{{image}}" { &full_wp_path } else { s })
            .collect();

        Command::new(command)
            .args(args)
            .spawn()
            .expect("command failed to start");
    }

    pub fn call_matugen(&self, full_wp_path: &str) {
        Command::new("matugen")
            .arg("image")
            .args(&self.matugen_flags)
            .arg(&format!(
                "{}/.cache/chameleon/thumbnails/{}",
                home_dir().unwrap().to_string_lossy(),
                cache_filename(Path::new(&full_wp_path), 160, 90)
            ))
            .spawn()
            .expect("command failed to start");
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
}

impl Provider for WallpapersProvider {
    fn name(&self) -> &'static str {
        "Wallpaper"
    }

    fn update_model(&self, query: &str, provider_changed: bool) {
        self.base.update_model(query, provider_changed);
    }

    fn len(&self) -> usize {
        self.base.len()
    }

    fn view(&self) -> gtk::ListBase {
        self.base.view()
    }

    fn invoke_action(&self) -> bool {
        let maybe_path = self
            .base
            .selection_model
            .selected_item()
            .and_downcast::<Wallpaper>();

        if let Some(path) = maybe_path {
            let full_wp_path =
                format!("{}/{}", self.wallpapers_path, path.picture_name());

            self.call_change_wallpaper_cmd(&full_wp_path);

            if self.matugen_enabled {
                self.call_matugen(&full_wp_path);
            }

            true
        } else {
            false
        }
    }

    fn reset_selection(&self) {
        self.base.reset_selection();
    }

    fn move_selection(&self, direction: Direction) {
        let model = &self.base.selection_model;
        let i = model.selected();
        let len = self.len() as u32;

        let new_i = match direction {
            Direction::Up if i >= 5 => i - 5,
            Direction::Right if i + 1 < len => i + 1,
            Direction::Down if i + 5 < len => i + 5,
            Direction::Left if i > 0 => i - 1,
            _ => i,
        };

        self.base.select(new_i);
    }
}
