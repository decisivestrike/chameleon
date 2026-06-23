use crate::config::ContentRules;
use crate::notification::{NotificationData, Urgency};
use gtk::gdk::MemoryTexture;
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Orientation, pango};

mod imp {
    use super::*;
    use gtk::Align;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct NotificationContentImp {
        pub icon: RefCell<Option<gtk::Image>>,
        pub title: RefCell<gtk::Label>,
        pub body: RefCell<Option<gtk::Label>>,
        pub text_container: RefCell<gtk::Box>,
        pub close_button: gtk::Button,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationContentImp {
        const NAME: &'static str = "NotificationContent";
        type Type = super::NotificationContent;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for NotificationContentImp {
        fn constructed(&self) {
            self.parent_constructed();

            self.close_button.set_valign(Align::Start);
            self.close_button.set_halign(Align::End);
            self.close_button.set_label("x");
            self.close_button
                .set_widget_name("notification-close-button");
            self.close_button.connect_clicked({
                let content = self.obj().clone();
                move |_| {
                    content
                        .parent()
                        .and_then(|p| p.downcast::<gtk::Window>().ok())
                        .map(|w| w.set_visible(false));
                }
            });
        }
    }

    impl WidgetImpl for NotificationContentImp {}

    impl BoxImpl for NotificationContentImp {}
}

glib::wrapper! {
    pub struct NotificationContent(ObjectSubclass<imp::NotificationContentImp>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Buildable, gtk::Accessible, gtk::ConstraintTarget;
}

impl NotificationContent {
    const NAME: &str = "notification";

    pub fn new(data: &NotificationData, content_rules: &ContentRules) -> Self {
        let obj: Self = Object::builder().build();
        obj.init_content(data, content_rules);
        obj
    }

    fn init_content(
        &self,
        data: &NotificationData,
        content_rules: &ContentRules,
    ) {
        let urgency = data.hints.urgency.unwrap_or(Urgency::Low);
        let urgency_class = urgency.as_ref();

        self.set_orientation(Orientation::Horizontal);
        self.set_halign(gtk::Align::Center);
        self.set_valign(gtk::Align::Center);
        self.set_baseline_position(gtk::BaselinePosition::Center);
        self.set_widget_name(Self::NAME);
        self.add_css_class(urgency_class);

        let text_container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .baseline_position(gtk::BaselinePosition::Center)
            .name("text-container")
            .build();

        let title = gtk::Label::builder()
            .label(&data.title)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .hexpand(true)
            .vexpand(true)
            .name("summary")
            .build();
        text_container.append(&title);

        let body = if !data.body.is_empty() {
            let body_label = gtk::Label::builder()
                .label(&data.body)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .name("body")
                .wrap(true)
                .hexpand(true)
                .vexpand(true)
                .wrap_mode(pango::WrapMode::Word)
                .build();
            text_container.append(&body_label);
            Some(body_label)
        } else {
            None
        };

        let imp = self.imp();
        imp.title.replace(title);
        imp.body.replace(body);
        imp.text_container.replace(text_container.clone());

        self.append(&text_container);

        let icon =
            match (data.app_icon.as_str(), data.hints.image_data.as_ref()) {
                ("", None) => None,
                ("", Some(image_data)) => {
                    let texture = MemoryTexture::from(image_data);
                    let icon = gtk::Image::builder()
                        .paintable(&texture)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .name("icon")
                        .pixel_size(content_rules.icon_size.into())
                        .build();

                    self.prepend(&icon);
                    Some(icon)
                }
                (filename, _) => {
                    let icon = gtk::Image::builder()
                        .file(filename)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .name("icon")
                        .pixel_size(96)
                        .build();
                    self.prepend(&icon);

                    Some(icon)
                }
            };

        imp.icon.replace(icon);

        self.append(&imp.close_button);
    }
}
