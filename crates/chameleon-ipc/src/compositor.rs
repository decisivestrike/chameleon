use tokio::sync::broadcast;

pub enum CompositorEvent {}

pub trait Compositor {
    fn subscribe(&self) -> broadcast::Receiver<CompositorEvent>;
}
