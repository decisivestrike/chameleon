use crate::window::NotificationWindow;
use gtk::glib::{SourceId, clone};
use gtkio::MAIN_CONTEXT;
use gtkio::time::timeout_local;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;

type WindowsMapInner = IndexMap<u32, Entry>;

/// Indexmap for windows with ttl
#[derive(Clone, Default)]
pub struct WindowsMap {
    indexmap: Rc<RefCell<WindowsMapInner>>,
}

impl WindowsMap {
    pub fn insert(
        &mut self,
        id: u32,
        window: NotificationWindow,
        ttl: Duration,
    ) {
        self.indexmap
            .borrow_mut()
            .insert(id, Entry::new(id, window, ttl, &self.indexmap));
    }

    /// With ttl update
    pub fn modify<F>(&mut self, key: u32, f: F) -> bool
    where
        F: FnOnce(&mut NotificationWindow),
    {
        let mut indexmap = self.indexmap.borrow_mut();
        let maybe_value = indexmap.get_mut(&key);

        if let Some(entry) = maybe_value {
            f(&mut entry.window);
            entry.restart_timer();
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: u32) -> Option<NotificationWindow> {
        if let Some(entry) = self.indexmap.borrow_mut().shift_remove(&key) {
            Some(entry.window.clone())
        } else {
            None
        }
    }

    /// Returns `true` if a window with the given id exists.
    pub fn contains_id(&self, id: u32) -> bool {
        self.indexmap.borrow().contains_key(&id)
    }

    /// Returns the number of stored windows.
    pub fn len(&self) -> usize {
        self.indexmap.borrow().len()
    }
}

struct Entry {
    id: u32,
    weak_map: Weak<RefCell<WindowsMapInner>>,
    ttl: Duration,
    timer_id: SourceId,
    window: NotificationWindow,
}

impl Entry {
    fn new(
        id: u32,
        window: NotificationWindow,
        ttl: Duration,
        map: &Rc<RefCell<WindowsMapInner>>,
    ) -> Self {
        let weak_map = Rc::downgrade(map);

        Self {
            id,
            timer_id: Self::create_timer(&weak_map, id, ttl),
            weak_map,
            ttl,
            window,
        }
    }

    fn create_timer(
        weak_map: &Weak<RefCell<WindowsMapInner>>,
        id: u32,
        ttl: Duration,
    ) -> SourceId {
        timeout_local(
            ttl,
            clone!(
                #[strong]
                weak_map,
                #[strong]
                id,
                move || {
                    if let Some(map) = weak_map.upgrade() {
                        map.borrow_mut().shift_remove(&id);
                    }
                }
            ),
        )
    }

    fn restart_timer(&mut self) {
        self.abort_timer();
        self.timer_id = Self::create_timer(&self.weak_map, self.id, self.ttl);
    }

    fn abort_timer(&self) {
        if let Some(source) = MAIN_CONTEXT.find_source_by_id(&self.timer_id) {
            source.destroy();
        }
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.abort_timer();
    }
}
