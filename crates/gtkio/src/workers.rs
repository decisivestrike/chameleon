use crate::future::spawn_local;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::error::RecvError as BroadcastRecvError;
use tokio::sync::watch::Sender as WatchSender;
use tokio::sync::{mpsc, watch};
use tracing::error;

pub trait MpscWorker<T>
where
    T: 'static,
{
    fn listen_local<F>(self, f: impl FnMut(Option<T>) -> F + 'static)
    where
        F: Future<Output = ()> + Send + 'static;
}

impl<T: 'static> MpscWorker<T> for mpsc::Receiver<T> {
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

pub trait BroadcastWorker<T>
where
    T: 'static,
{
    fn listen_local<F>(
        &self,
        f: impl FnMut(Result<T, BroadcastRecvError>) -> F + 'static,
    ) where
        F: Future<Output = ()> + Send + 'static;
}

impl<T: Clone + 'static> BroadcastWorker<T> for BroadcastSender<T> {
    fn listen_local<F>(
        &self,
        mut f: impl FnMut(Result<T, BroadcastRecvError>) -> F + 'static,
    ) where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut receiver = self.subscribe();

        spawn_local(async move {
            loop {
                let message = receiver.recv().await;
                f(message).await;
            }
        });
    }
}

pub trait WatchWorker<T>
where
    T: 'static,
{
    fn listen_local(
        &self,
        f: impl AsyncFnMut(&mut watch::Receiver<T>) + 'static,
    );
}

impl<T: 'static> WatchWorker<T> for WatchSender<T> {
    fn listen_local(
        &self,
        mut f: impl AsyncFnMut(&mut watch::Receiver<T>) + 'static,
    ) {
        let mut state = self.subscribe();

        spawn_local(async move {
            loop {
                if let Err(e) = state.changed().await {
                    error!("{e}");
                } else {
                    f(&mut state).await;
                }
            }
        });
    }
}
