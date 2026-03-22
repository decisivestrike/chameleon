use grapes::{
    Component,
    gtk::{self, Orientation},
    prelude::{BoxExt, Cast, WidgetExt},
};

use crate::entry_object::EntryInfo;

#[derive(Debug, Component)]
pub struct Card {
    icon: gtk::Image,
    name: gtk::Label,
    comment: gtk::Label,

    #[root]
    container: gtk::Box,
}

impl Card {
    pub fn new() -> Self {
        let icon = gtk::Image::builder()
            .icon_size(gtk::IconSize::Large)
            .pixel_size(48)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .build();

        let name = gtk::Label::builder()
            .xalign(0.0)
            .css_classes(["name"])
            .build();

        let comment = gtk::Label::builder()
            .wrap(true)
            .xalign(0.0)
            .overflow(gtk::Overflow::Hidden)
            .css_classes(["comment"])
            .build();

        let text_container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(["text-container"])
            .build();

        text_container.append(&name);
        text_container.append(&comment);

        let container = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .css_classes(["app"])
            .build();

        container.append(&icon);
        container.append(&text_container);

        Self {
            icon,
            name,
            comment,
            container,
        }
    }

    pub fn setup(&self, entry: &EntryInfo) {
        self.icon.set_icon_name(Some(&entry.icon()));
        self.name.set_text(&entry.name());
        self.comment.set_text(&entry.comment());
    }
}

impl From<gtk::Box> for Card {
    fn from(container: gtk::Box) -> Self {
        let icon = container
            .first_child()
            .expect("Icon should exist")
            .downcast::<gtk::Image>()
            .expect("First child should be Image");

        let text_container = container
            .last_child()
            .expect("Text container should exist")
            .downcast::<gtk::Box>()
            .expect("Second child should be Box");

        let name = text_container
            .first_child()
            .expect("Name label should exist")
            .downcast::<gtk::Label>()
            .expect("First text child should be Label");

        let comment = text_container
            .last_child()
            .expect("Comment label should exist")
            .downcast::<gtk::Label>()
            .expect("Second text child should be Label");

        Self {
            icon,
            name,
            comment,
            container,
        }
    }
}
