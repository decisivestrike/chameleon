pub mod applications;
pub mod wallpapers;

pub mod factory;
pub mod utils;

use gtk::glib::object::IsA;
use gtk::glib::{self};
// use gtk::prelude::ListItemExt;
// use gtk::{
//     CustomFilter, CustomSorter, FilterChange, FilterListModel, ListItem,
//     ListScrollFlags, ListView, SignalListItemFactory, SingleSelection,
//     SortListModel, SorterChange, gio,
// };
// use nucleo::Matcher;
// use std::cell::{Cell, RefCell};
// use std::marker::PhantomData;

pub trait Provider {
    fn name(&self) -> &'static str;

    /// Filter -> Sort
    fn update_model(&self, query: &str);

    fn view(&self) -> gtk::ListBase;

    fn invoke_action(&self) -> bool;

    fn reset(&self);
}

// Allows to bind data
pub trait Bindable: IsA<gtk::Widget> {
    type Data: IsA<glib::Object>;

    fn bind_data(&self, data: &Self::Data);
}

// pub struct ProviderBase<Data, Widget>
// where
//     Data: IsA<glib::Object>,
//     Widget: Bindable<Data = Data>,
// {
//     name: String,
//     factory: SignalListItemFactory,
//     pub store: gio::ListStore,
//     pub filter: CustomFilter,
//     pub sorter: CustomSorter,
//     pub selection_model: SingleSelection,
//     pub view: ListView,

//     pub last_query_len: Cell<usize>,

//     pub matcher: RefCell<Matcher>,

//     _data: PhantomData<Data>,
//     _widget: PhantomData<Widget>,
// }

// impl<Data, Widget> ProviderBase<Data, Widget>
// where
//     Data: IsA<glib::Object>,
//     Widget: Bindable<Data = Data>,
// {
//     pub fn new() -> Self {}

//     fn name(&self) -> &String {
//         &self.name
//     }

//     /// Filter -> Sort
//     fn update_model(&self, query: &str) {
//         let mut matcher = self.matcher.borrow_mut();

//         for entry in self.store.iter::<Data>().map(|e| e.unwrap()) {
//             let score = matcher
//                 .fuzzy_match(
//                     Utf32Str::Ascii(entry.name().as_bytes()),
//                     Utf32Str::Ascii(query.as_bytes()),
//                 )
//                 .map(|score| score as i32)
//                 .unwrap_or(-1);

//             entry.set_fuzzy_score(score);
//         }

//         let query_len = query.len();

//         if query_len < self.last_query_len.get() {
//             self.filter.changed(FilterChange::LessStrict);
//             self.sorter.changed(SorterChange::LessStrict);
//         } else {
//             self.filter.changed(FilterChange::MoreStrict);
//             self.sorter.changed(SorterChange::MoreStrict);
//         }

//         self.last_query_len.set(query_len);
//         self.reset()
//     }

//     fn view(&self) -> gtk::ListBase {}

//     fn invoke_action(&self) -> bool {}

//     fn reset(&self) {}
// }
