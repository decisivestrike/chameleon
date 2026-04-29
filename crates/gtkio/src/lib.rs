use futures::channel::oneshot;
use glib::{JoinHandle as GlibHandle, SourceId};
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle as TokioHandle;

static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("cant create runtime"));

pub fn spawn<F>(future: F) -> TokioHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    RUNTIME.spawn(future)
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn sum_callback() {
        spawn_with_local_callback(async { 2 + 2 }, |sum| assert_eq!(sum, 4));
    }
}

pub fn worker<T, F>(
    buffer: usize,
    f: impl FnOnce(mpsc::Sender<T>) -> F,
) -> mpsc::Receiver<T>
where
    T: Clone + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(buffer);

    spawn(f(sender));

    receiver
}

pub struct BroadcastWorker<T>
where
    T: Clone + 'static,
{
    sender: broadcast::Sender<T>,
    #[allow(dead_code)]
    handle: TokioHandle<()>,
}

impl<T: Clone + 'static> BroadcastWorker<T> {
    pub fn listen_local<F>(
        &self,
        mut f: impl FnMut(Result<T, RecvError>) -> F + 'static,
    ) where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();

        spawn_local(async move {
            loop {
                let message = receiver.recv().await;
                f(message).await;
            }
        });
    }
}

pub fn broadcast_worker<T, F>(
    capacity: usize,
    f: impl FnOnce(broadcast::Sender<T>) -> F,
) -> BroadcastWorker<T>
where
    T: Clone + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    let sender = broadcast::Sender::new(capacity);
    let handle = spawn(f(sender.clone()));

    BroadcastWorker { sender, handle }
}

pub fn timeout_local(delay: Duration, f: impl FnOnce() + 'static) -> SourceId {
    if delay.subsec_nanos() == 0 {
        let secs = delay.as_secs() as u32;
        glib::timeout_add_seconds_local_once(secs, f)
    } else {
        glib::timeout_add_local_once(delay, f)
    }
}
