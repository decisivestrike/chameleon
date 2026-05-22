use super::ApplicationRow;
use crate::entry_object::ApplicationEntry;
use gtk::prelude::*;
use gtk::{ListItem, SignalListItemFactory};

#[derive(Debug)]
pub struct Factory;

impl Factory {
    pub fn new() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let card = ApplicationRow::new();

            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&card));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            let entry_info = list_item
                .item()
                .and_downcast::<ApplicationEntry>()
                .expect("The item has to be an EntryInfo");

            let card = list_item
                .child()
                .and_downcast::<ApplicationRow>()
                .expect("The child has to be a Card");

            card.set_data(&entry_info);
        });

        factory
    }
}
