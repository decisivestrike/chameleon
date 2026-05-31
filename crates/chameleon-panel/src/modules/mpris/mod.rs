use crate::services::mpris::{CLIENT, MusicClient, PlayerUpdate};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use gtkio::RUNTIME;
use std::cell::RefCell;
use tokio::sync::mpsc;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MprisImp {
        label: RefCell<gtk::Label>,
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

            // Настройка внешнего вида
            widget.set_orientation(gtk::Orientation::Horizontal);
            widget.set_spacing(6);

            let label = gtk::Label::new(None);
            label.set_ellipsize(pango::EllipsizeMode::End);
            label.set_halign(gtk::Align::Center);
            widget.append(&label);
            self.label.replace(label);

            let (sender, mut receiver) = mpsc::channel::<String>(1);
            let label = self.label.clone();

            glib::spawn_future_local(async move {
                loop {
                    if let Some(title) = receiver.recv().await {
                        label.borrow().set_label(&title);
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
                                    let title =
                                        track.title.unwrap_or_else(|| {
                                            "No Title".to_string()
                                        });

                                    let artist =
                                        track.artist.unwrap_or_else(|| {
                                            "Unknown".to_string()
                                        });

                                    let title_text =
                                        format!("{} - {}", title, artist);

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
