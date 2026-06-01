use crate::modules::PanelModule;
use gtk::gdk::{MemoryFormat, MemoryTexture};
use gtk::glib::object::Cast;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object, clone};
use gtk::prelude::{BoxExt, WidgetExt};
use system_tray::client::{Event, UpdateEvent};
use system_tray::item::IconPixmap;

mod imp {
    use gtk::glib;
    use gtk::prelude::{BoxExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtkio::RUNTIME;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use system_tray::client::Client;

    pub struct TrayImp {
        pub client: Client,
        pub items: RefCell<HashMap<String, gtk::Image>>,
    }

    impl Default for TrayImp {
        fn default() -> Self {
            Self {
                client: RUNTIME.block_on(Client::new()).unwrap(),
                items: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrayImp {
        const NAME: &'static str = "ChameleonPanelTray";
        type Type = super::Tray;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for TrayImp {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_widget_name("tray");
            obj.set_spacing(8);
        }
    }

    impl WidgetImpl for TrayImp {}

    impl BoxImpl for TrayImp {}
}

glib::wrapper! {
    /// Panel
    pub struct Tray(ObjectSubclass<imp::TrayImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PanelModule for Tray {
    type Rules = ();

    fn create(
        rules: Self::Rules,
        meta: std::rc::Rc<super::Metadata>,
    ) -> anyhow::Result<gtk::Widget> {
        let tray = Tray::new();

        Ok(tray.upcast())
    }
}

impl Tray {
    pub fn new() -> Self {
        let tray: Self = Object::builder().build();
        let mut receiver = tray.imp().client.subscribe();

        glib::spawn_future_local(clone!(
            #[strong]
            tray,
            async move {
                while let Ok(event) = receiver.recv().await {
                    // println!("{:#?}", event);

                    match event {
                        Event::Add(name, item) => {
                            if let Some(pixmaps) = item.icon_pixmap {
                                let icon = pixmaps_to_icon(&pixmaps);
                                tray.append(&icon);
                                tray.imp()
                                    .items
                                    .borrow_mut()
                                    .insert(name, icon);
                            }
                        }
                        Event::Update(name, event) => match event {
                            UpdateEvent::Icon {
                                icon_name: _,
                                icon_pixmap,
                            } => {
                                if let Some(pixmaps) = icon_pixmap {
                                    let texture = pixmaps_to_texture(&pixmaps);

                                    let items = tray.imp().items.borrow_mut();
                                    let image = items.get(&name);

                                    image.map(|i| {
                                        i.set_paintable(Some(&texture))
                                    });
                                }
                            }
                            _ => (),
                        },
                        Event::Remove(name) => {
                            let icon =
                                tray.imp().items.borrow_mut().remove(&name);

                            icon.map(|i| i.unparent());
                        }
                    };
                }
            }
        ));

        let mu = tray.imp().client.items();
        let initial_items = mu.lock().unwrap();

        for (item, _tray_menu) in initial_items.values() {
            if let Some(ref pixmaps) = item.icon_pixmap {
                let icon = pixmaps_to_icon(&pixmaps);
                tray.append(&icon);
            }
        }

        tray
    }
}

fn pixmaps_to_texture(pixmaps: &Vec<IconPixmap>) -> MemoryTexture {
    let pixmap = pixmaps
        .iter()
        .max_by(|a, b| a.width.cmp(&b.width))
        .unwrap()
        .clone();

    MemoryTexture::new(
        pixmap.width,
        pixmap.height,
        MemoryFormat::A8r8g8b8Premultiplied,
        &glib::Bytes::from_owned(pixmap.pixels),
        (pixmap.width * 4) as usize,
    )
}

fn pixmaps_to_icon(pixmaps: &Vec<IconPixmap>) -> gtk::Image {
    let texture = pixmaps_to_texture(pixmaps);

    gtk::Image::builder()
        .paintable(&texture)
        .pixel_size(16)
        .build()
}
