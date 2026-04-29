use std::sync::LazyLock;
use tokio::runtime::Runtime;

static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("cant create runtime"));

use glib::JoinHandle as GlibHandle;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio::task::JoinHandle as TokioHandle;

// pub trait JoinHandle<T>: Future<Output = T> {}

// impl<T> JoinHandle<T> for TokioHandle<T> {}

enum Handle<T> {
    Tokio(TokioHandle<T>),
    Glib(GlibHandle<T>),
}

impl<T: 'static> Future for Handle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match &mut *self {
            Handle::Tokio(handle) => Pin::new(handle)
                .poll(cx)
                .map(|value| value.expect("panic in task")),
            Handle::Glib(handle) => Pin::new(handle)
                .poll(cx)
                .map(|value| value.expect("panic in task")),
        }
    }
}

pub struct Task<T> {
    handle: Handle<T>,
}

impl<T> Task<T> {
    pub fn then<O>(self, f: impl FnOnce(T) -> O + Send + 'static) -> Task<O>
    where
        T: Send + 'static,
        O: Send + 'static,
    {
        let handle = self.handle;
        spawn(async move { f(handle.await) })
    }

    /// This can be called only from the thread where the main context is
    /// running
    pub fn then_local<O>(self, f: impl FnOnce(T) -> O + 'static) -> Task<O>
    where
        T: 'static,
        O: 'static,
    {
        let handle = self.handle;
        spawn_local(async move { f(handle.await) })
    }

    pub fn perform() {}
}

pub fn spawn<F>(future: F) -> Task<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    Task {
        handle: Handle::Tokio(RUNTIME.spawn(future)),
    }
}

/// This can be called only from the thread where the main context is running
pub fn spawn_local<F>(future: F) -> Task<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    Task {
        handle: Handle::Glib(glib::spawn_future_local(future)),
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use glib::MainLoop;

    fn mainloop() -> MainLoop {
        let context = glib::MainContext::new();
        glib::MainLoop::new(Some(&context), true)
    }

    #[test]
    fn mytest() {
        let _ = mainloop();

        spawn_local(async { 2 + 2 })
            .then_local(|sum| sum * 2)
            .then_local(|mul| assert_eq!(mul, 8));
    }

    #[test]
    fn tokio_chain() {
        spawn(async { 2 + 2 })
            .then(|sum| sum * 2)
            .then(|mul| assert_eq!(mul, 8));
    }
}

pub struct BroadcastWorker<T> {
    sender: broadcast::Sender<T>,
    handle: TokioHandle<()>,
}

impl<T> BroadcastWorker<T> {
    pub fn listen_local<F>(&self, f: impl FnOnce(broadcast::Receiver<T>) -> F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let recv = self.sender.subscribe();
        spawn_local(f(recv));
    }

    // pub fn lazy<F>(f: impl FnOnce(broadcast::Sender<T>) -> F) ->
    // LazyLock<Self> where
    //     F: Future<Output = ()> + Send + 'static,
    // {
    //     LazyLock::new(|| broadcast_worker(f))
    // }
}

pub fn broadcast_worker<T, F>(
    f: impl FnOnce(broadcast::Sender<T>) -> F,
) -> BroadcastWorker<T>
where
    F: Future<Output = ()> + Send + 'static,
{
    let sender = broadcast::Sender::new(64);
    let handle = RUNTIME.spawn(f(sender.clone()));

    BroadcastWorker { sender, handle }
}
