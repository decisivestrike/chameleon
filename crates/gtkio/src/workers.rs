use crate::future::{spawn, spawn_local};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{self};
use tokio::sync::mpsc;
use tokio::task::JoinHandle as TokioHandle;

pub trait WorkerExt<T>
where
    T: 'static,
{
    fn listen_local<F>(self, f: impl FnMut(Option<T>) -> F + 'static)
    where
        F: Future<Output = ()> + Send + 'static;
}

impl<T: 'static> WorkerExt<T> for mpsc::Receiver<T> {
    fn listen_local<F>(mut self, mut f: impl FnMut(Option<T>) -> F + 'static)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        spawn_local(async move {
            loop {
                let message = self.recv().await;
                f(message).await;
            }
        });
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
