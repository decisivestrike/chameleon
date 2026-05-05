use futures::channel::oneshot;
use glib::JoinHandle as GlibHandle;
use std::fmt;
use tokio::task::JoinHandle as TokioHandle;

use crate::RUNTIME;

pub fn spawn<F>(future: F) -> TokioHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    RUNTIME.spawn(future)
}

pub fn spawn_blocking<F, R>(func: F) -> TokioHandle<F::Output>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    RUNTIME.spawn_blocking(func)
}

/// This can be called only from the thread where the main context is running
pub fn spawn_local<F>(future: F) -> GlibHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    glib::spawn_future_local(future)
}

/// For ui updates
pub fn spawn_with_local_callback<T>(
    f: impl Future<Output = T> + Send + 'static,
    callback: impl FnOnce(T) + 'static,
) -> GlibHandle<()>
where
    T: fmt::Debug + Send + 'static,
{
    let (sender, receiver) = oneshot::channel();

    spawn(async move {
        let value = f.await;
        sender.send(value).unwrap();
    });

    spawn_local(async move {
        let value = receiver.await.unwrap();
        callback(value);
    })
}
