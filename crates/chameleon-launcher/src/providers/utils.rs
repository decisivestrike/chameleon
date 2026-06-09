use gtk::glib::object::{Cast, IsA};
use gtk::{CustomFilter, CustomSorter, glib};
use std::any::type_name;
use std::cmp::Ordering;

fn type_error_msg<T>() -> String {
    format!("The object needs to be of type `{}`.", type_name::<T>())
}

/// Type safe wrapper for [`CustomFilter`]
pub fn filter<T>(f: impl Fn(&T) -> bool + 'static) -> CustomFilter
where
    T: IsA<glib::Object>,
{
    CustomFilter::new(move |obj| {
        let item = obj.downcast_ref::<T>().expect(&type_error_msg::<T>());

        f(item)
    })
}

/// Type safe wrapper for [`CustomSorter`]
pub fn sorter<T>(f: impl Fn(&T, &T) -> Ordering + 'static) -> CustomSorter
where
    T: IsA<glib::Object>,
{
    CustomSorter::new(move |obj_1, obj_2| {
        let first = obj_1.downcast_ref::<T>().expect(&type_error_msg::<T>());
        let second = obj_2.downcast_ref::<T>().expect(&type_error_msg::<T>());

        f(first, second).into()
    })
}
