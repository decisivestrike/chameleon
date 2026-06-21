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
    use std::time::Duration;

    use super::*;
    use crate::modules::mpris::player_window::PlayerWindow;
    use crate::services::mpris::PlayerState;
    use gtk::gdk::{MemoryTexture, Texture};
    use gtk::gdk_pixbuf::Pixbuf;
    use tracing::{error, info, warn};

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

            let (sender, mut receiver) = mpsc::channel::<Option<PlayerData>>(2);
            let mpris = self.obj().clone();

            glib::spawn_future_local(async move {
                loop {
                    if let Some(maybe_data) = receiver.recv().await {
                        if let Some(data) = maybe_data {
                            mpris.imp().label.set_label(&format!(
                                "{} - {}",
                                data.title, data.artist
                            ));

                            let window = &mpris.imp().window.imp();
                            window.title.set_label(&data.title);
                            window.artist.set_label(&data.artist);
                            window.album.set_label(
                                &data.album.unwrap_or(String::new()),
                            );

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
                                && let Ok(pixbuf) = Pixbuf::from_file_at_scale(
                                    path, 128, 128, true,
                                )
                            {
                                let buffer =
                                    pixbuf.save_to_bufferv("png", &[]).unwrap();
                                let bytes = glib::Bytes::from_owned(buffer);
                                let texture =
                                    Texture::from_bytes(&bytes).unwrap();

                                window.cover.set_paintable(Some(&texture));
                            }
                        } else {
                            mpris.imp().label.set_label("");

                            let window = &mpris.imp().window.imp();
                            window.title.set_label("");
                            window.artist.set_label("");
                            window.album.set_label("");

                            let child = &mpris.imp().window.imp().play_icon;
                            mpris
                                .imp()
                                .window
                                .imp()
                                .play_pause
                                .set_child(Some(child));

                            window.cover.set_paintable(None::<&MemoryTexture>);
                        }
                    }
                }
            });

            RUNTIME.spawn(async move {
                let mut receiver = CLIENT.subscribe_change();
                let mut previous_track_title = String::new();

                loop {
                    if let Ok(update) = receiver.recv().await {
                        match update {
                            PlayerUpdate::Update(boxed_track, status) => {
                                info!("{:#?}\n{:#?}", boxed_track, status);
                                let maybe_track = *boxed_track;

                                if matches!(status.state, PlayerState::Stopped)
                                {
                                    sender.send(None).await.unwrap();
                                }

                                if let Some(track) = maybe_track
                                    && !matches!(
                                        status.state,
                                        PlayerState::Stopped
                                    )
                                    && let Some(title) = track.title
                                {
                                    let cover_path: Option<String> =
                                        if previous_track_title != title {
                                            tokio::time::sleep(
                                                Duration::from_millis(100),
                                            )
                                            .await;

                                            let player = CLIENT.get_player();
                                            if player.is_none() {
                                                error!("player is none");
                                                continue;
                                            }
                                            let player = player.unwrap();

                                            let metadata =
                                                player.get_metadata();
                                            if let Err(e) = metadata {
                                                error!("{}", e);
                                                continue;
                                            }
                                            let metadata = metadata.unwrap();

                                            let art_url = metadata.art_url();
                                            if art_url.is_none() {
                                                warn!("no art url");
                                                continue;
                                            }

                                            previous_track_title =
                                                title.clone();

                                            Some({
                                                let mut u = art_url
                                                    .unwrap()
                                                    .to_string();
                                                u.drain(..7);
                                                u
                                            })
                                        } else {
                                            previous_track_title =
                                                title.clone();
                                            None
                                        }
                                        .or_else(|| {
                                            track.cover_path.map(|mut path| {
                                                path.drain(..7);
                                                path
                                            })
                                        });

                                    info!("{:?}", cover_path);

                                    let data = PlayerData {
                                        title,
                                        artist: track.artist.unwrap_or_else(
                                            || "Unknown".to_string(),
                                        ),
                                        album: track.album,
                                        cover_path,
                                        is_playing:
                                            if let PlayerState::Playing =
                                                status.state
                                            {
                                                true
                                            } else {
                                                false
                                            },
                                    };

                                    sender.send(Some(data)).await.unwrap();
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
