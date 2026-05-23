use gtk::glib::object::{Cast, CastNone, IsA};
use gtk::glib::{self};
use gtk::prelude::ListItemExt;
use gtk::{ListItem, SignalListItemFactory};

pub struct Factory;

impl Factory {
    pub fn new<Data, Widget>() -> SignalListItemFactory
    where
        Data: IsA<glib::Object>,
        Widget: super::Bindable<Data = Data> + Default,
    {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, obj| {
            let widget = Widget::default();

            obj.downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&widget));
        });

        factory.connect_bind(move |_, obj| {
            let list_item = obj
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            let data = list_item
                .item()
                .and_downcast::<Data>()
                .expect("The item has to be an Data");

            let widget = list_item
                .child()
                .and_downcast::<Widget>()
                .expect("Needs to be Widget");

            widget.bind_data(&data);
        });

        factory
    }
}
