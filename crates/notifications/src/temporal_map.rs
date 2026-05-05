use gtk::glib::{self, SourceId, clone};
use gtkio::MAIN_CONTEXT;
use gtkio::time::timeout_local;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::time::Duration;

/// Indexmap with ttl
#[derive(Clone, Default)]
pub struct TemporalMap<K, V>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
{
    indexmap: Rc<RefCell<IndexMap<K, (SourceId, V)>>>,
}

impl<K, V> TemporalMap<K, V>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
{
    pub fn insert(&mut self, key: K, value: V, ttl: Duration) {
        let timeout_id = timeout_local(
            ttl,
            clone!(
                #[weak(rename_to = indexmap)]
                self.indexmap,
                #[strong]
                key,
                move || {
                    indexmap.borrow_mut().shift_remove(&key);
                }
            ),
        );

        self.indexmap.borrow_mut().insert(key, (timeout_id, value));
    }

    pub fn modify(&mut self, key: K, f: impl FnOnce(&mut V)) -> bool {
        let mut indexmap = self.indexmap.borrow_mut();
        let maybe_value = indexmap.get_mut(&key);

        if let Some(value) = maybe_value {
            f(value);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some((timer_id, value)) =
            self.indexmap.borrow_mut().shift_remove(&key)
        {
            if let Some(source) = MAIN_CONTEXT.find_source_by_id(&timer_id) {
                source.destroy();
            }

            Some(value)
        } else {
            None
        }
    }
}
