use crate::window::NotificationWindow;
use gtk::glib::{self, SourceId, clone};
use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtke::WindowComponent;
use gtkio::MAIN_CONTEXT;
use gtkio::time::timeout_local;
use indexmap::IndexMap;
use layer_shell::{Edge, LayerShell};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use tracing::debug;

type WindowsMapInner = IndexMap<u32, Entry>;

/// Indexmap for windows with ttl
#[derive(Clone)]
pub struct WindowsMap {
    indexmap: Rc<RefCell<WindowsMapInner>>,
    max_count: usize,
    spacing: usize,
}

impl WindowsMap {
    pub fn new(max_count: usize, spacing: usize) -> Self {
        Self {
            indexmap: Default::default(),
            max_count,
            spacing,
        }
    }

    pub fn insert(
        &mut self,
        id: u32,
        window: NotificationWindow,
        ttl: Duration,
    ) {
        let mut notifications_count = self.len();

        if self.len() >= self.max_count {
            self.indexmap
                .borrow_mut()
                .shift_remove_index(0)
                .expect("should exists");

            notifications_count -= 1;
        }

        for (index, entry) in self.indexmap.borrow().iter().enumerate() {
            let notification = entry.1;

            let height = notification.window().height();
            let count = (notifications_count + 1 - index) as i32;

            let gaps = self.spacing as i32 * count;
            let heights = height * (count - 1);
            let margin_top = gaps + heights;

            notification.window().set_margin(Edge::Top, margin_top);
        }

        window.present();

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
        glib::timeout_add_local_once(
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

    fn window(&self) -> gtk::Window {
        self.window.window()
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.abort_timer();
        self.window().destroy();
        debug!("Drop entry with id = {}", self.id);
    }
}
