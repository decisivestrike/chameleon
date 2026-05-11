use crate::config::ContentRules;
use crate::notification::{NotificationData, Urgency};
use gtk::gdk::MemoryTexture;
use gtk::prelude::BoxExt;
use gtk::{Orientation, pango};
use gtke::Component;

#[derive(Clone, Component)]
pub struct NotificationContent {
    pub icon: Option<gtk::Image>,
    pub title: gtk::Label,
    pub body: Option<gtk::Label>,
    pub text_container: gtk::Box,

    #[root]
    pub container: gtk::Box,
}

impl NotificationContent {
    const NAME: &str = "notification";

    pub fn new(data: &NotificationData, content_rules: &ContentRules) -> Self {
        let urgency = data.hints.urgency.unwrap_or(Urgency::Low);
        let urgency_class = urgency.as_ref();

        let text_container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .hexpand(true)
            .vexpand(true)
            .baseline_position(gtk::BaselinePosition::Center)
            .name("text-container")
            .build();

        let title = gtk::Label::builder()
            .label(&data.title)
            .name("summary")
            .build();
        text_container.append(&title);

        let body = if !data.body.is_empty() {
            let body = gtk::Label::builder()
                .label(&data.body)
                .name("body")
                .wrap(true)
                .wrap_mode(pango::WrapMode::Word)
                .build();

            text_container.append(&body);

            Some(body)
        } else {
            None
        };

        let container = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .baseline_position(gtk::BaselinePosition::Center)
            .name(Self::NAME)
            .css_classes([urgency_class])
            .build();

        container.append(&text_container);

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

                    container.prepend(&icon);

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

                    container.prepend(&icon);

                    Some(icon)
                }
            };

        Self {
            title,
            icon,
            body,
            text_container,
            container,
        }
    }
}
