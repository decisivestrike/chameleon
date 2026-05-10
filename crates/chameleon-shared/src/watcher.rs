use futures::StreamExt;
use gtk::glib::{self, clone};
use gtke::Css;
use gtke::css::StylePriority;
use gtkio::future::spawn;
use inotify::{Inotify, WatchDescriptor, WatchMask};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use tracing::info;

pub struct FilesWatcher {
    inotify: Inotify,
    actions: HashMap<WatchDescriptor, FileAction>,
}

impl FilesWatcher {
    pub fn new() -> io::Result<Self> {
        let watcher = Self {
            inotify: Inotify::init()?,
            actions: Default::default(),
        };

        Ok(watcher)
    }

    pub fn add(mut self, path: PathBuf, f: FileActionFn) -> io::Result<Self> {
        let wd = self.inotify.watches().add(&path, WatchMask::CLOSE_WRITE)?;

        let action = FileAction::new(path, f);
        self.actions.insert(wd, action);

        Ok(self)
    }

    pub fn add_stylesheet(self, path: PathBuf) -> io::Result<Self> {
        self.add(
            path,
            Box::new(|path| {
                glib::idle_add_once(clone!(
                    #[strong]
                    path,
                    move || {
                        Css::load(&path).apply(StylePriority::User);
                        info!("Styles updated");
                    }
                ));
            }),
        )
    }

    pub fn run(self) {
        spawn(self.watcher());
    }

    async fn watcher(self) {
        let Self { inotify, actions } = self;

        let mut buffer = [0; 1024];
        let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

        loop {
            if let Some(maybe_event) = stream.next().await
                && let Ok(event) = maybe_event
                && let Some(file_action) = actions.get(&event.wd)
            {
                file_action.call();
            }
        }
    }
}

type FileActionFn = Box<dyn Fn(&PathBuf) + Send + 'static>;

struct FileAction {
    path: PathBuf,
    f: FileActionFn,
}

impl FileAction {
    fn new(path: PathBuf, f: FileActionFn) -> Self {
        Self { path, f }
    }

    fn call(&self) {
        (self.f)(&self.path)
    }
}
