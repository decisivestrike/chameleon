use super::ApplicationRow;
use crate::providers::applications::entry::ApplicationEntry;
use gtk::prelude::*;
use gtk::{ListItem, SignalListItemFactory};

#[derive(Debug)]
pub struct Factory;

impl Factory {
    pub fn new() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, obj| {
            let card = ApplicationRow::new();

            obj.downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&card));
        });

        factory.connect_bind(move |_, obj| {
            let list_item = obj
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            let entry_info = list_item
                .item()
                .and_downcast::<ApplicationEntry>()
                .expect("The item has to be an EntryInfo");

            let app_row = list_item
                .child()
                .and_downcast::<ApplicationRow>()
                .expect("The child has to be a Card");

            app_row.set_data(&entry_info);
        });

        factory
    }
}
