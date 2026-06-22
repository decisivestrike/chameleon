use crate::Css;
use crate::css::StylePriority;
use futures::StreamExt;
use gtk::glib::{self, clone};
use gtkio::future::spawn;
use inotify::{Inotify, WatchDescriptor, WatchMask};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct FilesWatcher {
    inotify: Inotify,
    actions: HashMap<WatchDescriptor, FileAction>,
    token: CancellationToken,
}

impl Default for FilesWatcher {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl FilesWatcher {
    pub fn new() -> io::Result<Self> {
        let watcher = Self {
            inotify: Inotify::init()?,
            actions: Default::default(),
            token: CancellationToken::new(),
        };

        Ok(watcher)
    }

    /// Add without move
    pub fn add2(&mut self, path: PathBuf, f: FileActionFn) -> io::Result<()> {
        let wd = self.inotify.watches().add(&path, WatchMask::CLOSE_WRITE)?;

        let action = FileAction::new(path, f);
        self.actions.insert(wd, action);

        Ok(())
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

    pub fn run(self) -> CancellationToken {
        let token = self.token.clone();
        spawn(self.watcher());

        token
    }

    async fn watcher(self) {
        let Self {
            inotify,
            actions,
            token,
        } = self;

        let mut buffer = [0; 1024];
        let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

        loop {
            tokio::select! {
                Some(maybe_event) = stream.next() => {
                    if let Ok(event) = maybe_event &&
                        let Some(file_action) = actions.get(&event.wd) {
                        file_action.call();
                    }
                }
                _ = token.cancelled() => {
                    break
                }
            }
        }
    }
}

type FileActionFn = Box<dyn Fn(&PathBuf) + Send + Sync + 'static>;

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
