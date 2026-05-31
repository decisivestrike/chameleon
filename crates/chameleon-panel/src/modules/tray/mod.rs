use crate::modules::PanelModule;
use gtk::gdk::{MemoryFormat, MemoryTexture};
use gtk::glib::object::Cast;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object, clone};
use gtk::prelude::BoxExt;
use system_tray::client::Event;
use system_tray::item::StatusNotifierItem;

mod imp {
    use gtk::glib;
    use gtk::prelude::{BoxExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtkio::RUNTIME;
    use system_tray::client::Client;

    pub struct TrayImp {
        pub client: Client,
    }

    impl Default for TrayImp {
        fn default() -> Self {
            Self {
                client: RUNTIME.block_on(Client::new()).unwrap(),
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
                        Event::Add(_, item) => {
                            let icon = item_to_icon(&item);
                            tray.append(&icon);
                        }
                        Event::Update(_, _) => (),
                        Event::Remove(_) => (),
                    };
                }
            }
        ));

        let mu = tray.imp().client.items();
        let initial_items = mu.lock().unwrap();

        for (item, _tray_menu) in initial_items.values() {
            let icon = item_to_icon(item);
            tray.append(&icon);
        }

        tray
    }
}

fn item_to_icon(item: &StatusNotifierItem) -> gtk::Image {
    let pixmaps = item.icon_pixmap.as_ref().unwrap();
    let pixmap = pixmaps
        .iter()
        .max_by(|a, b| a.width.cmp(&b.width))
        .unwrap()
        .clone();

    let texture = MemoryTexture::new(
        pixmap.width,
        pixmap.height,
        MemoryFormat::A8r8g8b8Premultiplied,
        &glib::Bytes::from_owned(pixmap.pixels),
        (pixmap.width * 4) as usize,
    );

    gtk::Image::builder()
        .paintable(&texture)
        .pixel_size(16)
        .build()
}
