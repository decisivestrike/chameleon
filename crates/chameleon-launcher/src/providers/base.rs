use gtk::gio::prelude::{ListModelExt, ListModelExtManual};
use gtk::prelude::{FilterExt, SorterExt};
use gtk::{
    CustomFilter, CustomSorter, FilterChange, FilterListModel, ListScrollFlags,
    SingleSelection, SortListModel, SorterChange, gio,
};
use nucleo::{Matcher, Utf32Str};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::marker::PhantomData;

use crate::providers::ItemData;
use crate::providers::utils::{filter, sorter};
use crate::providers::view::View;

pub struct ProviderBase<D>
where
    D: ItemData,
{
    pub store: gtk::gio::ListStore,
    pub filter: CustomFilter,
    pub sorter: CustomSorter,
    pub selection_model: SingleSelection,
    pub view: View,

    last_query_len: Cell<usize>,
    matcher: RefCell<Matcher>,
    _data: PhantomData<D>,
}

impl<D> ProviderBase<D>
where
    D: ItemData,
{
    pub fn new(
        store: gio::ListStore,
        f: impl FnOnce(&SingleSelection) -> View,
    ) -> Self {
        let filter = filter::<D>(|entry| entry.fuzzy_score() != -1);
        let filter_model = FilterListModel::builder()
            .model(&store)
            .filter(&filter)
            .build();

        let sorter = sorter::<D>(move |first, second| {
            let first_score = first.fuzzy_score();
            let second_score = second.fuzzy_score();

            match second_score.cmp(&first_score) {
                Ordering::Equal => {
                    let first_id = first.id();
                    let second_id = second.id();

                    first_id.cmp(&second_id)
                }
                score => score,
            }
        });
        let sort_model =
            SortListModel::new(Some(filter_model), Some(sorter.clone()));

        let selection_model = SingleSelection::new(Some(sort_model));
        let view = f(&selection_model);

        Self {
            store,
            filter,
            sorter,
            selection_model,
            view,
            last_query_len: Cell::new(0),
            matcher: Matcher::default().into(),
            _data: PhantomData,
        }
    }

    pub fn update_model(&self, query: &str) {
        self.update_fuzzy_score(query);
        self.update_filter_and_sorter(query);
        self.reset_selection();
    }

    pub fn len(&self) -> usize {
        self.selection_model.n_items() as usize
    }

    pub fn view(&self) -> gtk::ListBase {
        self.view.upcast()
    }

    pub fn reset_selection(&self) {
        if self.len() > 0 {
            self.selection_model.set_selected(0);
            self.view.scroll_to(0, ListScrollFlags::SELECT, None);
        }
    }

    fn update_fuzzy_score(&self, query: &str) {
        let mut matcher = self.matcher.borrow_mut();

        for item in self.store.iter::<D>().map(|e| e.unwrap()) {
            let id = item.id();
            let uft32_id = Utf32Str::Ascii(id.as_bytes());
            let uft32_query = Utf32Str::Ascii(query.as_bytes());

            let score = matcher
                .fuzzy_match(uft32_id, uft32_query)
                .map(|score| score as i32)
                .unwrap_or(-1);

            item.set_fuzzy_score(score);
        }
    }

    fn update_filter_and_sorter(&self, query: &str) {
        let query_len = query.len();

        if query_len < self.last_query_len.get() {
            self.filter.changed(FilterChange::LessStrict);
            self.sorter.changed(SorterChange::LessStrict);
        } else {
            self.filter.changed(FilterChange::MoreStrict);
            self.sorter.changed(SorterChange::MoreStrict);
        }

        self.last_query_len.set(query_len);
    }
}
