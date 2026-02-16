use grapes::tokio::sync::broadcast;

pub trait Compositor {
    type Message;

    fn subscribe(&self) -> broadcast::Receiver<Self::Message>;
}
