use gtk::gdk::{self, MemoryTexture};
use gtk::glib::{self, Object, clone};
use gtkio::RUNTIME;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::path::PathBuf;
use tokio::sync::oneshot;
use tracing::{error, info};

mod imp {
    use super::*;
    use gtk::gdk;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Wallpaper)]
    pub struct WallpaperImp {
        #[property(get, set)]
        texture: RefCell<Option<gdk::Texture>>,

        #[property(get, set)]
        picture_name: RefCell<String>,

        #[property(get, set, default = -1)]
        fuzzy_score: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperImp {
        const NAME: &'static str = "ChameleonLauncherWallpaperPath";
        type Type = super::Wallpaper;
    }

    #[glib::derived_properties]
    impl ObjectImpl for WallpaperImp {}
}

glib::wrapper! {
    pub struct Wallpaper(ObjectSubclass<imp::WallpaperImp>);
}

impl Wallpaper {
    pub fn new(path: PathBuf) -> Self {
        let wallpaper: Self = Object::builder().build();

        let os_filename = path.file_name().unwrap();
        wallpaper.set_picture_name(os_filename.to_string_lossy());

        let (sender, receiver) = oneshot::channel();

        RUNTIME.spawn_blocking(move || {
            let os_filename = path.file_name().unwrap();
            let name = os_filename.to_string_lossy();
            info!("Loading image: {}", name);

            let img = match ImageReader::open(&path).unwrap().decode() {
                Ok(img) => img,
                Err(e) => {
                    error!("Image '{}': {}", name, e);
                    return;
                }
            };

            let width = 160;
            let height = 90;

            let aspect_ratio = width as f32 / height as f32;

            let resized = Self::resize_by_crop(&img, aspect_ratio);
            let preview =
                resized.resize_exact(width, height, FilterType::Triangle);

            let texture = Self::img_to_texture(&preview);

            sender.send(texture).unwrap();
        });

        glib::spawn_future_local(clone!(
            #[weak]
            wallpaper,
            async move {
                if let Ok(texture) = receiver.await {
                    wallpaper.set_texture(texture);
                }
            }
        ));

        wallpaper
    }

    fn img_to_texture(img: &DynamicImage) -> MemoryTexture {
        let rgba = img.to_rgba8();
        let bytes = glib::Bytes::from_owned(rgba.into_raw());

        MemoryTexture::new(
            img.width() as i32,
            img.height() as i32,
            gdk::MemoryFormat::R8g8b8a8Premultiplied,
            &bytes,
            img.width() as usize * 4,
        )
    }

    fn resize_by_crop(img: &DynamicImage, aspect_ratio: f32) -> DynamicImage {
        let (w, h) = img.dimensions();
        let current_ratio = w as f32 / h as f32;

        if (current_ratio - aspect_ratio).abs() < f32::EPSILON {
            return img.clone();
        }

        if current_ratio > aspect_ratio {
            let new_w = ((h as f32 * aspect_ratio).round() as u32).min(w);
            let x = (w - new_w) / 2;

            img.crop_imm(x, 0, new_w, h)
        } else {
            let new_h = ((w as f32 / aspect_ratio).round() as u32).min(h);
            let y = (h - new_h) / 2;

            img.crop_imm(0, y, w, new_h)
        }
    }
}
