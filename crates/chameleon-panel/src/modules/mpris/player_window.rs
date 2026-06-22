use chameleon_shared::CHAMELEON_ROOT;
use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;
use layer_shell::{KeyboardMode, Layer, LayerShell};
use std::sync::LazyLock;

static ASSETS_ROOT_SVG: LazyLock<String> = LazyLock::new(|| {
    CHAMELEON_ROOT
        .join("assets")
        .join("svg")
        .to_string_lossy()
        .to_string()
});

mod imp {
    use super::*;
    use crate::services::mpris::{CLIENT, MusicClient};
    use gtk::subclass::prelude::*;
    use gtk::{Align, ContentFit, Orientation, glib};
    use layer_shell::Edge;
    use tracing::error;

    pub struct PlayerWindowImp {
        pub title: gtk::Label,
        pub artist: gtk::Label,
        pub album: gtk::Label,
        pub cover: gtk::Picture,
        root: gtk::Box,

        pub play_pause: gtk::Button,

        pub pause_icon: gtk::Image,
        pub play_icon: gtk::Image,
    }

    impl Default for PlayerWindowImp {
        fn default() -> Self {
            let cover = gtk::Picture::builder()
                .name("player-cover")
                .halign(Align::Center)
                .valign(Align::Center)
                .content_fit(ContentFit::Fill)
                .width_request(128)
                .height_request(128)
                .build();

            let content = gtk::Box::builder()
                .halign(Align::Center)
                .valign(Align::Fill)
                .orientation(Orientation::Vertical)
                .spacing(0)
                .build();
            content.set_widget_name("player-content");

            let root = gtk::Box::new(Orientation::Horizontal, 0);
            root.append(&cover);
            root.append(&content);

            let album = gtk::Label::new(None);
            album.set_widget_name("player-album");
            album.set_xalign(0.0);
            album.set_halign(Align::Start);
            content.append(&album);

            let title = gtk::Label::new(None);
            title.set_widget_name("player-title");
            title.set_xalign(0.0);
            title.set_halign(Align::Start);
            content.append(&title);

            let artist = gtk::Label::new(None);
            artist.set_widget_name("player-artist");
            artist.set_xalign(0.0);
            artist.set_halign(Align::Start);
            content.append(&artist);

            let buttons = gtk::Box::new(Orientation::Horizontal, 4);
            buttons.set_widget_name("player-buttons");
            buttons.set_halign(Align::Center);
            buttons.set_vexpand(true);
            buttons.set_valign(Align::End);
            content.append(&buttons);

            let previous = gtk::Button::new();
            previous.connect_clicked(|_| {
                if let Err(e) = CLIENT.prev() {
                    error!("{}", e);
                }
            });
            let p = String::new() + &*ASSETS_ROOT_SVG + "/skip_previous.svg";
            println!("{}", p);
            let skip_previous_icon =
                gtk::Image::builder().file(p).pixel_size(24).build();
            previous.set_child(Some(&skip_previous_icon));
            buttons.append(&previous);

            let play_pause = gtk::Button::new();
            play_pause.connect_clicked(|_| {
                if let Err(e) = CLIENT.toggle_play_pause() {
                    error!("{}", e);
                }
            });
            let pause_icon = gtk::Image::builder()
                .file(String::new() + &*ASSETS_ROOT_SVG + "/pause.svg")
                .pixel_size(24)
                .build();

            let play_icon = gtk::Image::builder()
                .file(String::new() + &*ASSETS_ROOT_SVG + "/play.svg")
                .pixel_size(24)
                .build();

            play_pause.set_child(Some(&pause_icon));
            buttons.append(&play_pause);

            let next = gtk::Button::new();
            next.connect_clicked(|_| {
                if let Err(e) = CLIENT.next() {
                    error!("{}", e);
                }
            });
            let skip_next_icon = gtk::Image::builder()
                .file(String::new() + &*ASSETS_ROOT_SVG + "/skip_next.svg")
                .pixel_size(24)
                .build();
            next.set_child(Some(&skip_next_icon));
            buttons.append(&next);

            Self {
                cover,
                title,
                artist,
                album,
                root,
                play_pause,
                pause_icon,
                play_icon,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayerWindowImp {
        const NAME: &'static str = "ChameleonPlayer";
        type Type = super::PlayerWindow;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for PlayerWindowImp {
        fn constructed(&self) {
            self.parent_constructed();
            let window = self.obj();

            window.set_focusable(true);
            window.set_vexpand(false);
            window.set_hexpand(false);
            window.set_default_size(-1, -1); // Auto size
            window.set_widget_name("player-window");
            window.set_child(Some(&self.root));

            window.init_layer_shell();
            window.set_namespace(Some("chameleon-player"));
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Top, 16);
            window.set_layer(Layer::Top);
            window.set_keyboard_mode(KeyboardMode::None);
        }
    }

    impl WidgetImpl for PlayerWindowImp {}

    impl WindowImpl for PlayerWindowImp {}
}

glib::wrapper! {
    pub struct PlayerWindow(ObjectSubclass<imp::PlayerWindowImp>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PlayerWindow {
    pub fn new() -> Self {
        let player_window: Self = Object::builder().build();

        player_window
    }

    pub fn toggle_visibility(&self) {
        let target_visibility = !self.get_visible();

        if target_visibility {
            self.present();
        } else {
            self.set_visible(target_visibility);
        }
    }
}
