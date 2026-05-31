use crate::services::mpris::{CLIENT, MusicClient, PlayerUpdate};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use gtkio::RUNTIME;
use tokio::sync::mpsc;

mod imp {
    use super::*;
    use tracing::error;

    #[derive(Default)]
    pub struct MprisImp {
        label: gtk::Label,
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

            let widget = self.obj();

            widget.set_orientation(gtk::Orientation::Horizontal);
            widget.set_spacing(6);

            let lmb_play_pause = gtk::GestureClick::new();
            lmb_play_pause.set_button(1);
            lmb_play_pause.connect_pressed(|_gesture, _n_press, _x, _y| {
                if let Err(e) = CLIENT.toggle_play_pause() {
                    error!("Toggle play/pause error: {}", e);
                }
            });
            self.label.add_controller(lmb_play_pause);

            let rmb_next_track = gtk::GestureClick::new();
            rmb_next_track.set_button(3);
            rmb_next_track.connect_pressed(|_gesture, _n_press, _x, _y| {
                if let Err(e) = CLIENT.next() {
                    error!("Next track error: {}", e);
                }
            });
            self.label.add_controller(rmb_next_track);

            self.label.set_ellipsize(pango::EllipsizeMode::End);
            self.label.set_halign(gtk::Align::Center);

            widget.append(&self.label);

            let (sender, mut receiver) = mpsc::channel::<String>(1);
            let label = self.label.clone();

            glib::spawn_future_local(async move {
                loop {
                    if let Some(title) = receiver.recv().await {
                        label.set_label(&title);
                    }
                }
            });

            RUNTIME.spawn(async move {
                let mut receiver = CLIENT.subscribe_change();

                loop {
                    if let Ok(update) = receiver.recv().await {
                        match update {
                            PlayerUpdate::Update(boxed_track, _status) => {
                                let maybe_track = *boxed_track;

                                if let Some(track) = maybe_track {
                                    let title_text =
                                        if let Some(title) = track.title {
                                            let artist =
                                                track.artist.unwrap_or_else(
                                                    || "Unknown".to_string(),
                                                );

                                            format!("{} - {}", title, artist)
                                        } else {
                                            String::new()
                                        };

                                    sender.send(title_text).await.unwrap();
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
        @extends gtk::Widget, gtk::Box,
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
