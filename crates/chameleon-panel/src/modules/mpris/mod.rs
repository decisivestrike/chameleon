mod player_window;

use crate::services::mpris::{CLIENT, MusicClient, PlayerUpdate};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use gtkio::RUNTIME;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct PlayerData {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub cover_path: Option<String>,
    pub is_playing: bool,
}

mod imp {
    use super::*;
    use crate::modules::mpris::player_window::PlayerWindow;
    use crate::services::mpris::PlayerState;
    use gtk::gdk::Texture;
    use gtk::gdk_pixbuf::Pixbuf;
    use tracing::info;

    pub struct MprisImp {
        label: gtk::Label,
        window: PlayerWindow,
    }

    impl Default for MprisImp {
        fn default() -> Self {
            let label = gtk::Label::new(None);
            let window = PlayerWindow::new();

            Self { label, window }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MprisImp {
        const NAME: &'static str = "ChameleonMprisPlayer";
        type Type = Mpris;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for MprisImp {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.append(&self.label);
            obj.set_orientation(gtk::Orientation::Horizontal);
            obj.set_spacing(6);

            self.label.set_ellipsize(pango::EllipsizeMode::End);
            self.label.set_halign(gtk::Align::Center);

            let toggle_controller = gtk::GestureClick::new();
            toggle_controller.set_button(1);
            toggle_controller.connect_pressed({
                let player_window = self.window.clone();
                move |_gesture, _n_press, _x, _y| {
                    player_window.toggle_visibility();
                }
            });
            self.label.add_controller(toggle_controller);

            let (sender, mut receiver) = mpsc::channel::<PlayerData>(2);
            let mpris = self.obj().clone();

            glib::spawn_future_local(async move {
                loop {
                    if let Some(data) = receiver.recv().await {
                        mpris.imp().label.set_label(&format!(
                            "{} - {}",
                            data.title, data.artist
                        ));

                        let window = &mpris.imp().window.imp();
                        window.title.set_label(&data.title);
                        window.artist.set_label(&data.artist);
                        window
                            .album
                            .set_label(&data.album.unwrap_or(String::new()));

                        let child = if data.is_playing {
                            mpris.imp().window.imp().pause_icon.clone()
                        } else {
                            mpris.imp().window.imp().play_icon.clone()
                        };
                        mpris
                            .imp()
                            .window
                            .imp()
                            .play_pause
                            .set_child(Some(&child));

                        if let Some(path) = data.cover_path
                            && let Ok(pixbuf) =
                                Pixbuf::from_file_at_scale(path, 128, 128, true)
                        {
                            let buffer =
                                pixbuf.save_to_bufferv("png", &[]).unwrap();
                            let bytes = glib::Bytes::from_owned(buffer);
                            let texture = Texture::from_bytes(&bytes).unwrap();

                            window.cover.set_paintable(Some(&texture));
                        }
                    }
                }
            });

            RUNTIME.spawn(async move {
                let mut receiver = CLIENT.subscribe_change();

                loop {
                    if let Ok(update) = receiver.recv().await {
                        match update {
                            PlayerUpdate::Update(boxed_track, status) => {
                                info!("{:#?}", boxed_track);
                                let maybe_track = *boxed_track;

                                if let Some(track) = maybe_track
                                    && let Some(title) = track.title
                                {
                                    let data = PlayerData {
                                        title,
                                        artist: track.artist.unwrap_or_else(
                                            || "Unknown".to_string(),
                                        ),
                                        album: track.album,
                                        cover_path: if let Some(mut path) =
                                            track.cover_path
                                        {
                                            path.drain(..7);
                                            Some(path)
                                        } else {
                                            None
                                        },
                                        is_playing:
                                            if let PlayerState::Playing =
                                                status.state
                                            {
                                                true
                                            } else {
                                                false
                                            },
                                    };

                                    sender.send(data).await.unwrap();
                                }
                            }
                            _ => (),
                        };
                    }
                }
            });
        }
    }

    impl WidgetImpl for MprisImp {}

    impl BoxImpl for MprisImp {}
}

glib::wrapper! {
    pub struct Mpris(ObjectSubclass<imp::MprisImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Mpris {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl super::PanelModule for Mpris {
    type Rules = ();

    fn create(
        rules: Self::Rules,
        meta: std::rc::Rc<super::Metadata>,
    ) -> anyhow::Result<gtk::Widget> {
        let mpris = Mpris::new();

        Ok(mpris.upcast())
    }
}
