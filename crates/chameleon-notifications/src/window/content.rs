use crate::config::ContentRules;
use crate::notification::{NotificationData, Urgency};
use gtk::Orientation;
use gtk::gdk::MemoryTexture;
use gtk::prelude::BoxExt;
use gtke::Component;

#[derive(Clone, Component)]
pub struct NotificationContent {
    pub icon: Option<gtk::Image>,
    pub title: gtk::Label,
    pub body: gtk::Label,
    pub text_container: gtk::Box,

    #[root]
    pub container: gtk::Box,
}

impl NotificationContent {
    const NAME: &str = "notification";

    pub fn new(data: &NotificationData, content_rules: &ContentRules) -> Self {
        let title = gtk::Label::builder()
            .label(&data.title)
            .name("summary")
            .build();
        let body = gtk::Label::builder().label(&data.body).name("body").build();

        let urgency = data.hints.urgency.unwrap_or(Urgency::Low);
        let urgency_class = urgency.as_ref();

        let text_container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();

        text_container.append(&title);
        text_container.append(&body);

        let container = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .name(Self::NAME)
            .css_classes([urgency_class])
            .build();

        container.append(&text_container);

        let icon = if let Some(ref image_data) = data.hints.image_data {
            let texture = MemoryTexture::from(image_data);
            let icon = gtk::Image::builder()
                .paintable(&texture)
                .pixel_size(content_rules.icon_size.into())
                .build();

            container.prepend(&icon);

            Some(icon)
        } else {
            None
        };

        Self {
            title,
            icon,
            body,
            text_container,
            container,
        }
    }

    // pub fn set_icon(&mut self, texture: MemoryTexture) {
    //     if let Some(ref icon) = self.icon {
    //         icon.set_paintable(Some(&texture));
    //     } else {
    //         let icon = gtk::Image::builder()
    //             .paintable(&texture)
    //             .pixel_size(32 /* CONFIG.icon_size.into() */)
    //             .build();

    //         self.container.prepend(&icon);
    //         self.icon = Some(icon);
    //     }
    // }
}
