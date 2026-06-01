use crate::services::mpris::{CLIENT, MusicClient, PlayerUpdate};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use gtkio::RUNTIME;
use tokio::sync::mpsc;

mod imp {
    use super::*;
    use gtk::ContentFit;
    use gtk::gdk::Texture;
    use gtk::gdk_pixbuf::Pixbuf;
    use tracing::{error, info};

    pub struct MprisImp {
        label: gtk::Label,
        cover: gtk::Picture,
        popover: gtk::Popover,
        popover_content: gtk::Box,
    }

    impl Default for MprisImp {
        fn default() -> Self {
            let cover = gtk::Picture::builder()
                .content_fit(ContentFit::ScaleDown)
                .width_request(128)
                .height_request(128)
                .build();

            let label = gtk::Label::new(None);

            let popover_content = gtk::Box::new(gtk::Orientation::Vertical, 10);
            popover_content.append(&cover);

            let popover = gtk::Popover::builder().has_arrow(false).build();

            popover.set_child(Some(&popover_content));
            popover.unparent();
            popover.set_parent(&label);

            Self {
                label,
                cover,
                popover,
                popover_content,
            }
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

            let lmb_play_pause = gtk::GestureClick::new();
            lmb_play_pause.set_button(1);
            lmb_play_pause.connect_pressed(|_gesture, _n_press, _x, _y| {
                if let Err(e) = CLIENT.toggle_play_pause() {
                    error!("Toggle play/pause error: {}", e);
                }
            });
            self.label.add_controller(lmb_play_pause);

            let mmb_toggle_popover = gtk::GestureClick::new();
            mmb_toggle_popover.set_button(2);
            mmb_toggle_popover.connect_pressed({
                let popover_clone = self.popover.clone();
                move |_gesture, _n_press, _x, _y| {
                    popover_clone.popup();
                }
            });
            self.label.add_controller(mmb_toggle_popover);

            let rmb_next_track = gtk::GestureClick::new();
            rmb_next_track.set_button(3);
            rmb_next_track.connect_pressed(|_gesture, _n_press, _x, _y| {
                if let Err(e) = CLIENT.next() {
                    error!("Next track error: {}", e);
                }
            });
            self.label.add_controller(rmb_next_track);

            let (sender, mut receiver) =
                mpsc::channel::<(String, Option<String>)>(1);
            let tray = self.obj().clone();

            glib::spawn_future_local(async move {
                loop {
                    if let Some((title, cover_path)) = receiver.recv().await {
                        tray.imp().label.set_label(&title);

                        if let Some(path) = cover_path
                            && let Ok(pixbuf) =
                                Pixbuf::from_file_at_scale(path, 128, 128, true)
                        {
                            let buffer =
                                pixbuf.save_to_bufferv("png", &[]).unwrap();
                            let bytes = glib::Bytes::from_owned(buffer);
                            let texture = Texture::from_bytes(&bytes).unwrap();

                            tray.imp().cover.set_paintable(Some(&texture));
                        }
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

                                    let cover_path = {
                                        if let Some(mut path) = track.cover_path
                                        {
                                            path.drain(..7);

                                            Some(path)
                                        } else {
                                            None
                                        }
                                    };

                                    let message = (title_text, cover_path);

                                    info!("{:#?}", message);

                                    sender.send(message).await.unwrap();
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
