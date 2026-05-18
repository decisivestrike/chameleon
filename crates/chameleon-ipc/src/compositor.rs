use tokio::sync::broadcast;

pub enum CompositorEvent {}

pub trait Compositor {
    type Event;

    fn subscribe(&self) -> broadcast::Receiver<Self::Event>;
}
